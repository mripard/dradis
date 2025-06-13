#include <linux/v4l2-subdev.h>
#include <linux/videodev2.h>

/** Introduced with Linux 6.2 */
#define MEDIA_BUS_FMT_Y16_1X16			0x202e

/** Introduced with Linux 6.3 */
#define MEDIA_BUS_FMT_BGR666_1X18		0x1023
#define MEDIA_BUS_FMT_BGR666_1X24_CPADHI	0x1024
#define MEDIA_BUS_FMT_RGB565_1X24_CPADHI	0x1022

/** Introduced with Linux 6.7 */
#define MEDIA_BUS_FMT_RGB666_2X9_BE		0x1025

/** Introduced with Linux 6.10 */
#define MEDIA_BUS_FMT_META_8			0x8001
#define MEDIA_BUS_FMT_META_10			0x8002
#define MEDIA_BUS_FMT_META_12			0x8003
#define MEDIA_BUS_FMT_META_14			0x8004
#define MEDIA_BUS_FMT_META_16			0x8005
#define MEDIA_BUS_FMT_META_20			0x8006
#define MEDIA_BUS_FMT_META_24			0x8007

/** Introduced with Linux 6.13 */
#define MEDIA_BUS_FMT_RGB101010_1X7X5_SPWG	0x1026
#define MEDIA_BUS_FMT_RGB101010_1X7X5_JEIDA	0x1027

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
