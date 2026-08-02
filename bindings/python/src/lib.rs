mod adapter;
mod builtin;
mod client;
mod convert;
mod stream;

use pyo3::prelude::*;
use pyo3::{create_exception, exceptions::PyException};

create_exception!(_native, MaxtError, PyException);

#[pymodule]
#[pyo3(name = "_native")]
fn native(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add("MaxtError", module.py().get_type::<MaxtError>())?;
    module.add_class::<builtin::NativeUpbitAdapter>()?;
    module.add_class::<builtin::NativeBithumbAdapter>()?;
    module.add_class::<builtin::NativeBinanceAdapter>()?;
    module.add_class::<builtin::NativeBinanceListenKey>()?;
    module.add_class::<builtin::NativeHyperliquidAdapter>()?;
    module.add_class::<client::NativeClient>()?;
    module.add_class::<stream::NativeMarketStream>()?;
    module.add_class::<stream::NativeAccountStream>()?;
    Ok(())
}
