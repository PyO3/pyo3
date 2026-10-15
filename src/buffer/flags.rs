use super::PyBufferContiguity;

pub(super) const CONTIGUITY_UNDEFINED: u8 = PyBufferContiguity::Undefined as u8;
pub(super) const CONTIGUITY_C: u8 = PyBufferContiguity::C as u8;
pub(super) const CONTIGUITY_F: u8 = PyBufferContiguity::F as u8;
pub(super) const CONTIGUITY_ANY: u8 = PyBufferContiguity::Any as u8;

pub struct PyBufferFlags<
    const FORMAT: bool,
    const SHAPE: bool,
    const STRIDE: bool,
    const INDIRECT: bool,
    const WRITABLE: bool,
    const CONTIGUITY: u8,
>;

#[diagnostic::on_unimplemented(
    message = "format information has already been requested for this buffer request",
    note = "remove the extra `.format()` call"
)]
pub trait CanRequestFormat {}
#[diagnostic::do_not_recommend]
impl<
    const SHAPE: bool,
    const STRIDE: bool,
    const INDIRECT: bool,
    const WRITABLE: bool,
    const CONTIGUITY: u8,
> CanRequestFormat for PyBufferFlags<false, SHAPE, STRIDE, INDIRECT, WRITABLE, CONTIGUITY>
{
}

#[diagnostic::on_unimplemented(
    message = "shape information has already been requested for this buffer request",
    note = "remove the extra `.nd()` call"
)]
pub trait CanRequestShape {}
#[diagnostic::do_not_recommend]
impl<
    const FORMAT: bool,
    const STRIDE: bool,
    const INDIRECT: bool,
    const WRITABLE: bool,
    const CONTIGUITY: u8,
> CanRequestShape for PyBufferFlags<FORMAT, false, STRIDE, INDIRECT, WRITABLE, CONTIGUITY>
{
}

#[diagnostic::on_unimplemented(
    message = "stride information has already been requested for this buffer request",
    note = "remove the extra `.strides()` call"
)]
pub trait CanRequestStrides {}
#[diagnostic::do_not_recommend]
impl<
    const FORMAT: bool,
    const SHAPE: bool,
    const INDIRECT: bool,
    const WRITABLE: bool,
    const CONTIGUITY: u8,
> CanRequestStrides for PyBufferFlags<FORMAT, SHAPE, false, INDIRECT, WRITABLE, CONTIGUITY>
{
}

#[diagnostic::on_unimplemented(
    message = "suboffsets can only be requested on a direct unconstrained buffer request",
    note = "call `.indirect()` before any contiguity builder, and only once"
)]
pub trait CanRequestIndirect {}
#[diagnostic::do_not_recommend]
impl<const FORMAT: bool, const SHAPE: bool, const STRIDE: bool, const WRITABLE: bool>
    CanRequestIndirect
    for PyBufferFlags<FORMAT, SHAPE, STRIDE, false, WRITABLE, { CONTIGUITY_UNDEFINED }>
{
}

#[diagnostic::on_unimplemented(
    message = "writability has already been requested for this buffer request",
    note = "remove the extra `.writable()` call"
)]
pub trait CanRequestWritable {}
#[diagnostic::do_not_recommend]
impl<
    const FORMAT: bool,
    const SHAPE: bool,
    const STRIDE: bool,
    const INDIRECT: bool,
    const CONTIGUITY: u8,
> CanRequestWritable for PyBufferFlags<FORMAT, SHAPE, STRIDE, INDIRECT, false, CONTIGUITY>
{
}

#[diagnostic::on_unimplemented(
    message = "contiguity has already been constrained for this buffer request",
    note = "only one of `.c_contiguous()`, `.f_contiguous()`, or `.any_contiguous()` may be used"
)]
pub trait CanRequestContiguity {}
#[diagnostic::do_not_recommend]
impl<
    const FORMAT: bool,
    const SHAPE: bool,
    const STRIDE: bool,
    const INDIRECT: bool,
    const WRITABLE: bool,
> CanRequestContiguity
    for PyBufferFlags<FORMAT, SHAPE, STRIDE, INDIRECT, WRITABLE, { CONTIGUITY_UNDEFINED }>
{
}

#[diagnostic::on_unimplemented(
    message = "writability is not guaranteed by the requested buffer flags",
    note = "use `.writable()` when building a buffer request to guarantee writability"
)]
pub trait GuaranteesWritable {}
#[diagnostic::do_not_recommend]
impl<
    const FORMAT: bool,
    const SHAPE: bool,
    const STRIDE: bool,
    const INDIRECT: bool,
    const CONTIGUITY: u8,
> GuaranteesWritable for PyBufferFlags<FORMAT, SHAPE, STRIDE, INDIRECT, true, CONTIGUITY>
{
}

#[diagnostic::on_unimplemented(
    message = "C contiguity is not guaranteed by the requested buffer flags",
    note = "use `.c_contiguous()` when building a buffer request to guarantee C contiguity"
)]
pub trait GuaranteesCContiguous {}
#[diagnostic::do_not_recommend]
impl<
    const FORMAT: bool,
    const SHAPE: bool,
    const STRIDE: bool,
    const INDIRECT: bool,
    const WRITABLE: bool,
> GuaranteesCContiguous
    for PyBufferFlags<FORMAT, SHAPE, STRIDE, INDIRECT, WRITABLE, { CONTIGUITY_C }>
{
}

#[diagnostic::on_unimplemented(
    message = "Fortran contiguity is not guaranteed by the requested buffer flags",
    note = "use `.f_contiguous()` when building a buffer request to guarantee Fortran contiguity"
)]
pub trait GuaranteesFContiguous {}
#[diagnostic::do_not_recommend]
impl<
    const FORMAT: bool,
    const SHAPE: bool,
    const STRIDE: bool,
    const INDIRECT: bool,
    const WRITABLE: bool,
> GuaranteesFContiguous
    for PyBufferFlags<FORMAT, SHAPE, STRIDE, INDIRECT, WRITABLE, { CONTIGUITY_F }>
{
}

/// Marker trait for buffer flags which have requested format information.
#[diagnostic::on_unimplemented(
    message = "format information is not available with the requested buffer flags",
    note = "use `.format()` when building a buffer request to request format information",
    note = "`PyBufferRequest::simple()` and `PyBufferRequest::simple().writable()` also imply u8 format"
)]
pub trait IncludesFormat {
    const ASSUME_U8: bool;
}

#[diagnostic::do_not_recommend]
impl<
    const SHAPE: bool,
    const STRIDE: bool,
    const INDIRECT: bool,
    const WRITABLE: bool,
    const CONTIGUITY: u8,
> IncludesFormat for PyBufferFlags<true, SHAPE, STRIDE, INDIRECT, WRITABLE, CONTIGUITY>
{
    const ASSUME_U8: bool = false;
}

// Simple (maybe writable) buffers also have an implied u8 format.
#[diagnostic::do_not_recommend]
impl<const WRITABLE: bool> IncludesFormat
    for PyBufferFlags<false, false, false, false, WRITABLE, { CONTIGUITY_UNDEFINED }>
{
    const ASSUME_U8: bool = true;
}

#[diagnostic::on_unimplemented(
    message = "shape information is not available with the requested buffer flags",
    note = "use `.nd()` when building a buffer request to request shape information"
)]
pub trait IncludesShape {}
#[diagnostic::do_not_recommend]
impl<
    const FORMAT: bool,
    const STRIDE: bool,
    const INDIRECT: bool,
    const WRITABLE: bool,
    const CONTIGUITY: u8,
> IncludesShape for PyBufferFlags<FORMAT, true, STRIDE, INDIRECT, WRITABLE, CONTIGUITY>
{
}

#[diagnostic::on_unimplemented(
    message = "strides information is not available with the requested buffer flags",
    note = "use `.strides()` when building a buffer request to request stride information"
)]
pub trait IncludesStrides {}
#[diagnostic::do_not_recommend]
impl<
    const FORMAT: bool,
    const SHAPE: bool,
    const INDIRECT: bool,
    const WRITABLE: bool,
    const CONTIGUITY: u8,
> IncludesStrides for PyBufferFlags<FORMAT, SHAPE, true, INDIRECT, WRITABLE, CONTIGUITY>
{
}

#[diagnostic::on_unimplemented(
    message = "suboffsets information is not available with the requested buffer flags",
    note = "use `.indirect()` when building a buffer request to request suboffset information"
)]
pub trait IncludesSuboffsets {}
#[diagnostic::do_not_recommend]
impl<
    const FORMAT: bool,
    const SHAPE: bool,
    const STRIDE: bool,
    const WRITABLE: bool,
    const CONTIGUITY: u8,
> IncludesSuboffsets for PyBufferFlags<FORMAT, SHAPE, STRIDE, true, WRITABLE, CONTIGUITY>
{
}

pub trait Sealed {}
impl<
    const FORMAT: bool,
    const SHAPE: bool,
    const STRIDE: bool,
    const INDIRECT: bool,
    const WRITABLE: bool,
    const CONTIGUITY: u8,
> Sealed for PyBufferFlags<FORMAT, SHAPE, STRIDE, INDIRECT, WRITABLE, CONTIGUITY>
{
}

/// Trait implemented by all hidden [`super::PyBufferRequest`] states.
pub trait PyBufferRequestType: Sealed {
    /// Whether this is a simple request, which treats the buffer as unsigned bytes.
    const IS_SIMPLE: bool;

    /// The contiguity requirement encoded by these flags.
    const CONTIGUITY: PyBufferContiguity;

    /// Whether these flags require a writable buffer.
    const WRITABLE: bool;

    /// The state after requesting format information.
    type WithFormat: PyBufferRequestType + IncludesFormat;

    /// The state after requesting shape information.
    type WithShape: PyBufferRequestType + IncludesShape;

    /// The state after requesting strides information.
    type WithStrides: PyBufferRequestType + IncludesShape + IncludesStrides;

    /// The state after requesting indirect / suboffset information.
    type WithIndirect: PyBufferRequestType + IncludesShape + IncludesStrides + IncludesSuboffsets;

    /// The state after requesting writability.
    type WithWritable: PyBufferRequestType;

    /// The state after requesting C contiguity.
    type WithCContiguous: PyBufferRequestType;

    /// The state after requesting Fortran contiguity.
    type WithFContiguous: PyBufferRequestType;

    /// The state after requesting either C or Fortran contiguity.
    type WithAnyContiguous: PyBufferRequestType;
}

impl<
    const FORMAT: bool,
    const SHAPE: bool,
    const STRIDE: bool,
    const INDIRECT: bool,
    const WRITABLE: bool,
    const CONTIGUITY_REQ: u8,
> PyBufferRequestType for PyBufferFlags<FORMAT, SHAPE, STRIDE, INDIRECT, WRITABLE, CONTIGUITY_REQ>
{
    const IS_SIMPLE: bool = !FORMAT && !SHAPE;

    const CONTIGUITY: PyBufferContiguity = match CONTIGUITY_REQ {
        CONTIGUITY_UNDEFINED => PyBufferContiguity::Undefined,
        CONTIGUITY_C => PyBufferContiguity::C,
        CONTIGUITY_F => PyBufferContiguity::F,
        CONTIGUITY_ANY => PyBufferContiguity::Any,
        _ => panic!("invalid buffer contiguity"),
    };
    const WRITABLE: bool = WRITABLE;

    type WithFormat = PyBufferFlags<true, SHAPE, STRIDE, INDIRECT, WRITABLE, CONTIGUITY_REQ>;
    type WithShape = PyBufferFlags<FORMAT, true, STRIDE, INDIRECT, WRITABLE, CONTIGUITY_REQ>;
    type WithStrides = PyBufferFlags<FORMAT, true, true, INDIRECT, WRITABLE, CONTIGUITY_REQ>;
    type WithIndirect = PyBufferFlags<FORMAT, true, true, true, WRITABLE, CONTIGUITY_UNDEFINED>;
    type WithWritable = PyBufferFlags<FORMAT, SHAPE, STRIDE, INDIRECT, true, CONTIGUITY_REQ>;
    type WithCContiguous = PyBufferFlags<FORMAT, true, true, false, WRITABLE, CONTIGUITY_C>;
    type WithFContiguous = PyBufferFlags<FORMAT, true, true, false, WRITABLE, CONTIGUITY_F>;
    type WithAnyContiguous = PyBufferFlags<FORMAT, true, true, false, WRITABLE, CONTIGUITY_ANY>;
}
