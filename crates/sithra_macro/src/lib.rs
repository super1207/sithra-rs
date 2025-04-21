use proc_macro::TokenStream;
use quote::quote;
use syn::meta::ParseNestedMeta;
use syn::parse::Result;
use syn::parse_macro_input;
use syn::{FnArg, ItemFn, Pat, ReturnType};

#[derive(Default)]
struct EffectLoopArgs {
    subscribers: Option<syn::Expr>,
    state_type: Option<syn::Type>,
}

impl EffectLoopArgs {
    fn parse(&mut self, meta: ParseNestedMeta) -> Result<()> {
        if meta.path.is_ident("subscribers") {
            self.subscribers = Some(meta.value()?.parse()?);
            Ok(())
        } else if meta.path.is_ident("state") {
            self.state_type = Some(meta.value()?.parse()?);
            Ok(())
        } else {
            Err(meta.error("期望的参数是 `subscribers` 或 `state`"))
        }
    }
}

#[proc_macro_attribute]
pub fn subscribe_message(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let fn_body = &input_fn.block;
    let fn_vis = &input_fn.vis;

    let mut typed_args = Vec::new();
    for arg in &input_fn.sig.inputs {
        if let FnArg::Typed(pat_ty) = arg {
            typed_args.push(pat_ty);
        } else if let FnArg::Receiver(_) = arg {
            panic!("subscribe_message 宏不能用于方法");
        }
    }

    let (has_state_param, state_param_sig, event_param_sig, event_param_ident, state_ty) =
        match typed_args.len() {
            1 => {
                let event_arg = typed_args[0];
                let event_pat = &event_arg.pat;
                let event_ty = &event_arg.ty;
                let event_ident = if let Pat::Ident(pat_ident) = &**event_pat {
                    pat_ident.ident.clone()
                } else {
                    panic!("事件参数必须是标识符");
                };
                (
                    false,
                    None,
                    Some(quote! { #event_pat: #event_ty }),
                    Some(event_ident),
                    None,
                )
            }
            2 => {
                let state_arg = typed_args[0];
                let event_arg = typed_args[1];
                let state_pat = &state_arg.pat;
                let state_ty = &state_arg.ty;
                let event_pat = &event_arg.pat;
                let event_ty = &event_arg.ty;
                let event_ident = if let Pat::Ident(pat_ident) = &**event_pat {
                    pat_ident.ident.clone()
                } else {
                    panic!("事件参数必须是标识符");
                };
                // State 参数也使用原始类型
                (
                    true,
                    Some(quote! { #state_pat: #state_ty }),
                    Some(quote! { #event_pat: #event_ty }),
                    Some(event_ident),
                    Some(state_ty),
                )
            }
            _ => panic!("subscribe_message 宏仅支持 1 或 2 个参数"),
        };

    let event_param_name = event_param_ident.expect("内部错误：未能提取事件参数名称");

    // Build inner function signature parameters
    let inner_params = if let Some(state_sig) = state_param_sig {
        let event_sig = event_param_sig.expect("内部错误：事件参数签名丢失");
        quote! { #state_sig, #event_sig }
    } else {
        event_param_sig.expect("内部错误：事件参数签名丢失").into()
    };

    // Build inner function call arguments
    let inner_call_args = if has_state_param {
        quote! { _state.clone(), &flattened }
    } else {
        quote! { &flattened }
    };

    let return_type = match &input_fn.sig.output {
        ReturnType::Default => quote!(()),
        ReturnType::Type(_, ty) => quote!(#ty),
    };

    let expanded = if let Some(state_ty) = state_ty {
        quote! {
            #[::ioevent::subscriber]
            #fn_vis async fn #fn_name(_state: #state_ty, #event_param_name: ::sithra_common::event::MessageEvent) -> ::sithra_common::prelude::Result {
                async fn inner(
                    #inner_params
                ) -> #return_type {
                    #fn_body
                }

                let flattened = #event_param_name.flatten();
                let message = inner(#inner_call_args).await;
                if let Some(message) = message {
                    flattened.reply(&_state, message.clone()).await.unwrap();
                }
                Ok(())
            }
        }
    } else {
        quote! {
            #[::ioevent::subscriber]
            #fn_vis async fn #fn_name<S: ::sithra_common::state::SithraState>(_state: ::ioevent::State<S>, #event_param_name: ::sithra_common::event::MessageEvent) -> ::sithra_common::prelude::Result {
                async fn inner(
                    #inner_params
                ) -> #return_type {
                    #fn_body
                }

                let flattened = #event_param_name.flatten();
                let message = inner(#inner_call_args).await;
                if let Some(message) = message {
                    flattened.reply(&_state, message.clone()).await.unwrap();
                }
                Ok(())
            }
        }
    };

    TokenStream::from(expanded)
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
    let state_type = attrs.state_type.expect("缺少 state 参数");

    quote! {
        #[allow(unreachable_code)]
        #[::tokio::main]
        async fn main() {
            let args: Vec<String> = ::std::env::args().collect();
            let self_id = if args.len() > 1 {
                args[1].parse().unwrap_or_else(|_| {
                    ::log::error!("无效的 self_id 参数");
                    ::std::process::exit(1);
                })
            } else {
                ::log::error!("缺少 self_id 参数");
                ::std::process::exit(1);
            };

            let subscribes = ::ioevent::Subscribers::init(#subscribers);
            let mut builder = ::ioevent::BusBuilder::new(subscribes);
            builder.add_pair(::ioevent::IoPair::stdio());
            let (bus, wright) = builder.build();
            ::sithra_common::log::init_log(wright.clone(), ::log::LevelFilter::Info);
            let state = ::ioevent::State::new(<#state_type as ::sithra_common::state::SithraState>::create(self_id), wright.clone());

            let handle_bus = bus.run(state, &|e| {
                ::log::error!("总线错误: {:?}", e);
            }).await;

            let handle_main_loop = tokio::spawn(async move {
                _main(&wright).await;
            });

            handle_bus.join().await;
        }
        #[doc(hidden)]
        #output
    }
    .into()
}
