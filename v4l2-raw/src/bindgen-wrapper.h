#include <linux/v4l2-subdev.h>
#include <linux/videodev2.h>

/** <div rustbindgen attribute="#[derive(facet::Facet, facet_enum_repr::FacetEnumRepr)]" */
enum v4l2_buf_type;

/** <div rustbindgen attribute="#[derive(facet::Facet, facet_enum_repr::FacetEnumRepr)]" */
enum v4l2_colorspace;

/** <div rustbindgen attribute="#[derive(facet::Facet, facet_enum_repr::FacetEnumRepr)]" */
/** <div rustbindgen attribute="#[facet_enum_repr(panic_into(u8))]"></div> */
enum v4l2_field;

/** <div rustbindgen attribute="#[derive(facet::Facet, facet_enum_repr::FacetEnumRepr)]" */
enum v4l2_mbus_pixelcode;

/** <div rustbindgen attribute="#[derive(facet::Facet, facet_enum_repr::FacetEnumRepr)]" */
enum v4l2_memory;

/** <div rustbindgen attribute="#[derive(facet::Facet, facet_enum_repr::FacetEnumRepr)]" */
/** <div rustbindgen attribute="#[facet_enum_repr(panic_into(u16))]"></div> */
enum v4l2_quantization;

/** <div rustbindgen attribute="#[derive(facet::Facet, facet_enum_repr::FacetEnumRepr)]" */
enum v4l2_subdev_format_whence;

/**
 * The stream parameter got introduced with Linux 6.3. Use the newer structure
 * to have the right definition, and we'll runtime-check it.
 *
 * <div rustbindgen replaces="v4l2_subdev_format"></div>
 */
struct v4l2_subdev_format_newer {
	__u32 which;
	__u32 pad;
	struct v4l2_mbus_framefmt format;
	__u32 stream;
	__u32 reserved[7];
};

/** <div rustbindgen attribute="#[derive(facet::Facet, facet_enum_repr::FacetEnumRepr)]" */
/** <div rustbindgen attribute="#[facet_enum_repr(panic_into(u16))]"></div> */
enum v4l2_xfer_func;
