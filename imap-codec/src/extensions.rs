#[cfg(feature = "ext_compress")]
#[cfg_attr(docsrs, doc(cfg(feature = "ext_compress")))]
pub mod compress;
#[cfg(feature = "ext_enable")]
#[cfg_attr(docsrs, doc(cfg(feature = "ext_enable")))]
pub mod enable;
#[cfg(feature = "ext_idle")]
#[cfg_attr(docsrs, doc(cfg(feature = "ext_idle")))]
pub mod idle;
#[cfg(feature = "ext_literal")]
#[cfg_attr(docsrs, doc(cfg(feature = "ext_literal")))]
pub mod literal;
#[cfg(feature = "ext_move")]
#[cfg_attr(docsrs, doc(cfg(feature = "ext_move")))]
pub mod r#move;
#[cfg(feature = "ext_quota")]
#[cfg_attr(docsrs, doc(cfg(feature = "ext_quota")))]
pub mod quota;
#[cfg(feature = "ext_unselect")]
#[cfg_attr(docsrs, doc(cfg(feature = "ext_unselect")))]
pub mod unselect;
