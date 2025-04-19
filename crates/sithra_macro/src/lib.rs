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
pub fn main_loop(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::ItemFn);
    let fn_name = input.sig.ident.clone();

    let mut attrs = EffectLoopArgs::default();
    let parser = syn::meta::parser(|meta| attrs.parse(meta));
    parse_macro_input!(args with parser);

    let subscribers = attrs.subscribers.expect("缺少 subscribers 参数");
    let state = attrs.state.expect("缺少 state 参数");

    quote! {
        #[allow(unreachable_code)]
        #[tokio::main]
        async fn main() {
            let subscribes = ::ioevent::Subscribers::init(#subscribers);
            let mut builder = ::ioevent::BusBuilder::new(subscribes);
            builder.add_pair(::ioevent::IoPair::stdio());
            let ::ioevent::Bus {
                mut subscribe_ticker,
                mut effect_ticker,
                effect_wright,
            } = builder.build();
            let state = ::ioevent::State::new(#state, effect_wright.clone());
            let handle = ::tokio::spawn(async move {
                loop {
                    #fn_name(&effect_wright).await;
                }
            });
            loop {
                ::tokio::select! {
                    _ = subscribe_ticker.tick(&state) => {},
                    _ = effect_ticker.tick() => {},
                }
            }
            handle.abort();
        }
        #input
    }
    .into()
}
