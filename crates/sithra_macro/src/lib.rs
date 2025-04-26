use proc_macro::TokenStream;
use quote::quote;
use syn::meta::ParseNestedMeta;
use syn::parse::Result;
use syn::parse_macro_input;

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
