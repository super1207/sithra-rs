#[macro_export]
#[doc(hidden)]
macro_rules! into_response {
    ($path:expr, $type:ty) => {
        impl $crate::__private::sithra_server::response::IntoResponse for $type {
            fn into_response(self) -> $crate::__private::sithra_server::response::Response {
                $crate::__private::sithra_transport::datapack::RequestDataPack::default()
                    .path($path)
                    .payload(self)
                    .into_response()
            }
        }
    };
}
