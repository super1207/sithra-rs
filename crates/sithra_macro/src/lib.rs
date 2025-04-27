use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::meta::ParseNestedMeta;
use syn::parse::Result;
use syn::{parse_macro_input, FnArg, ItemFn};

#[derive(Default)]
struct EffectLoopArgs {
    subscribers: Option<syn::Expr>,
    state: Option<syn::Expr>,
}

impl EffectLoopArgs {
    fn parse(&mut self, meta: ParseNestedMeta) -> Result<()> {
        if meta.path.is_ident("subscribers") {
            self.subscribers = Some(meta.value()?.parse()?);
            Ok(())
        } else if meta.path.is_ident("state") {
            self.state = Some(meta.value()?.parse()?);
            Ok(())
        } else {
            Err(meta.error("期望的参数是 `subscribers` 或 `state`"))
        }
    }
}

#[proc_macro_attribute]
pub fn main(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::ItemFn);
    let mut output = input.clone();
    output.sig.ident = proc_macro2::Ident::new("_main", input.sig.ident.span());

    let mut attrs = EffectLoopArgs::default();
    let parser = syn::meta::parser(|meta| attrs.parse(meta));
    parse_macro_input!(args with parser);

    let subscribers = attrs.subscribers.expect("缺少 subscribers 参数");
    let state = attrs.state.expect("缺少 state 参数");

    quote! {
        #[allow(unreachable_code)]
        #[::tokio::main]
        async fn main() {
            let args: Vec<String> = ::std::env::args().collect();
            let (data_path,): (::std::path::PathBuf,) = if args.len() > 1 {
                (args[1].parse().unwrap_or_else(|_| {
                    ::log::error!("无效的 data_path 参数");
                    ::std::process::exit(1);
                }),)
            } else {
                ::log::error!("缺少 data_path 参数");
                ::std::process::exit(1);
            };
            ::sithra_common::global::set_data_path(data_path).unwrap_or_else(|e| {
                ::log::error!("设置数据路径失败: {:?}", e);
                ::std::process::exit(1);
            });

            let subscribes = ::ioevent::Subscribers::init(#subscribers);
            let mut builder = ::ioevent::BusBuilder::new(subscribes);
            builder.add_pair(::ioevent::IoPair::stdio());
            let (bus, wright) = builder.build();
            ::sithra_common::log::init_log(wright.clone(), ::log::LevelFilter::Info);
            ::log::set_max_level(::log::LevelFilter::Trace);
            let state = ::ioevent::State::new(#state, wright.clone());

            let handle_bus = bus.run(state, &|e| {
                ::log::error!("总线错误: {:?}", e);
                match e {
                    ::ioevent::error::BusError::BusRecv(ioevent::error::BusRecvError::Recv(ioevent::error::RecvError::Io(_))) => {
                        ::std::process::exit(1);
                    }
                    _ => {}
                }
            }).await;
            
            let (_, close_handle) = handle_bus.spawn();

            let handle_main_loop = tokio::spawn(async move {
                _main(&wright).await;
            });

            let _ = ::tokio::signal::ctrl_c().await;
            close_handle.close();
            handle_main_loop.abort();
            ::std::process::exit(0);
        }
        #[doc(hidden)]
        #output
    }
    .into()
}


#[proc_macro_attribute]
pub fn adapt_api(attr: TokenStream, item: TokenStream) -> TokenStream {
    let type_param = parse_macro_input!(attr as syn::Type);
    let original_fn = parse_macro_input!(item as ItemFn);

    if original_fn.sig.asyncness.is_none() {
        return quote! { compile_error!("procedure macro can only be applied to async functions"); }.into();
    }
    
    let params = original_fn.sig.inputs.iter().collect::<Vec<_>>();
    let (state_param, event_param) = match params.len() {
        1 => (None, params[0]),
        2 => (Some(params[0]), params[1]),
        _ => panic!("Expected 1 or 2 parameters"),
    };

    let (event_ty, event_name) = match event_param {
        FnArg::Typed(pat_type) => (&pat_type.ty, &pat_type.pat),
        _ => panic!("Event parameter must be a typed parameter"),
    };

    let state_ty_name = state_param.map(|param| match param {
        FnArg::Typed(pat_type) => (&pat_type.ty, &pat_type.pat),
        _ => panic!("State parameter must be a typed parameter"),
    });
    
    let raw_generics = &original_fn.sig.generics.type_params().map(|v|v.clone()).collect::<Vec<_>>();

    let (generics, new_params) = if let Some((state_ty, state_name)) = state_ty_name {
        let params = quote! {
            #state_name: &#state_ty,
            #event_name: &::ioevent::event::EventData,
        };
        (quote! { <#(#raw_generics),*> }, params)
    } else {
        let params = quote! {
            _state: &::ioevent::state::State<_STATE>,
            #event_name: &::ioevent::event::EventData,
        };
        (quote! { <#(#raw_generics),* _STATE: ::ioevent::state::ProcedureCallWright + ::std::clone::Clone + ::std::marker::Send + ::std::marker::Sync + 'static> }, params)
    };

    let event_try_into = quote! {
        let #event_name: ::core::result::Result<::ioevent::state::ProcedureCallData, ::ioevent::error::TryFromEventError> = ::std::convert::TryInto::try_into(#event_name);
    };

    let state_clone = if let Some((_, state_name)) = state_ty_name {
        quote! {
            let #state_name = ::std::clone::Clone::clone(#state_name);
        }
    } else {
        quote! {
            let _state = ::std::clone::Clone::clone(_state);
        }
    };

    let original_stmts = &original_fn.block.stmts;

    let async_block = if let Some((_, state_name)) = state_ty_name {
        quote! {
            async move {
                let #event_name = #event_name?;
                if <#event_ty as ::ioevent::state::ProcedureCallRequest>::match_self(&#event_name) {
                    let echo = #event_name.echo;
                    let #event_name = <#event_ty as ::std::convert::TryFrom<::ioevent::state::ProcedureCallData>>::try_from(#event_name)?;
                    if !#event_name.match_adapter::<#type_param>() {
                        return Ok(());
                    }
                    let response: ::core::result::Result<_, ::ioevent::error::CallSubscribeError> = {
                        #(#original_stmts)*
                    };
                    ::ioevent::state::ProcedureCallExt::resolve::<#event_ty>(&#state_name, echo, &response?).await?;
                }
                Ok(())
            }
        }
    } else {
        quote! {
            async move {
                let #event_name = #event_name?;
                if <#event_ty as ::ioevent::state::ProcedureCallRequest>::match_self(&#event_name) {
                    let echo = #event_name.echo;
                    let #event_name = <#event_ty as ::std::convert::TryFrom<::ioevent::state::ProcedureCallData>>::try_from(#event_name)?;
                    let response: ::core::result::Result<_, ::ioevent::error::CallSubscribeError> = {
                        #(#original_stmts)*
                    };
                    ::ioevent::state::ProcedureCallExt::resolve::<#event_ty>(&_state, echo, &response?).await?;
                }
                Ok(())
            }
        }
    };

    let func_name = &original_fn.sig.ident;
    let mod_name = format_ident!("{}", func_name);

    let vis = &original_fn.vis;

    let mod_block = quote! {
        #[doc(hidden)]
        #vis mod #mod_name {
            use super::*;
            pub type _Event = ::ioevent::state::ProcedureCallData;
        }
    };

    let expanded = quote! {
        #vis fn #func_name #generics (#new_params) -> ::ioevent::future::SubscribeFutureRet {
            #event_try_into
            #state_clone
            ::std::boxed::Box::pin(#async_block)
        }
        #mod_block
    };

    TokenStream::from(expanded)
}