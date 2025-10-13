/*
 * memcpy - copy memory area
 *
 * Copyright (c) 2019-2023, Arm Limited.
 * SPDX-License-Identifier: MIT OR Apache-2.0 WITH LLVM-exception
 */

/* Assumptions:
 *
 * ARMv8-a, AArch64, Advanced SIMD, unaligned accesses.
 *
 */

/*
 * Taken from
 * https://github.com/ARM-software/optimized-routines/blob/4fc7374ff35420f057b958bb1f5d9d450e664d8f/string/aarch64/memcpy-advsimd.S,
 */

	dstin	.req	x0
	src		.req	x1
	count	.req	x2
	dst		.req	x3
	srcend	.req	x4
	dstend	.req	x5
	A_l		.req	x6
	A_lw	.req	w6
	A_h		.req	x7
	B_l		.req	x8
	B_lw	.req	w8
	B_h		.req	x9
	C_lw	.req	w10
	tmp1	.req	x14

	A_q		.req	q0
	B_q		.req	q1
	C_q		.req	q2
	D_q		.req	q3
	E_q		.req	q4
	F_q		.req	q5
	G_q		.req	q6
	H_q		.req	q7

/* This implementation handles overlaps and supports both memcpy and memmove
   from a single entry point.  It uses unaligned accesses and branchless
   sequences to keep the code small, simple and improve performance.

   Copies are split into 3 main cases: small copies of up to 32 bytes, medium
   copies of up to 128 bytes, and large copies.  The overhead of the overlap
   check is negligible since it is only required for large copies.

   Large copies use a software pipelined loop processing 64 bytes per iteration.
   The source pointer is 16-byte aligned to minimize unaligned accesses.
   The loop tail is handled by always copying 64 bytes from the end.
*/

	.section ".text", "ax", %progbits
	.globl	__memcpy_aarch64_simd
	.globl	__memmove_aarch64_simd
__memcpy_aarch64_simd:
__memmove_aarch64_simd:
	add	srcend, src, count
	cmp	count, 128
	b.hi	.Lcopy_long_advsimd
	add	dstend, dstin, count
	cmp	count, 32
	b.hi	.Lcopy32_128_advsimd
	nop

	/* Small copies: 0..32 bytes.  */
	cmp	count, 16
	b.lo	.Lcopy16_advsimd
	ldr	A_q, [src]
	ldr	B_q, [srcend, -16]
	str	A_q, [dstin]
	str	B_q, [dstend, -16]
	ret

	.p2align 4
	/* Medium copies: 33..128 bytes.  */
.Lcopy32_128_advsimd:
	ldp	A_q, B_q, [src]
	ldp	C_q, D_q, [srcend, -32]
	cmp	count, 64
	b.hi	.Lcopy128_advsimd
	stp	A_q, B_q, [dstin]
	stp	C_q, D_q, [dstend, -32]
	ret

	.p2align 4
	/* Copy 8-15 bytes.  */
.Lcopy16_advsimd:
	tbz	count, 3, .Lcopy8_advsimd
	ldr	A_l, [src]
	ldr	A_h, [srcend, -8]
	str	A_l, [dstin]
	str	A_h, [dstend, -8]
	ret

	/* Copy 4-7 bytes.  */
.Lcopy8_advsimd:
	tbz	count, 2, .Lcopy4_advsimd
	ldr	A_lw, [src]
	ldr	B_lw, [srcend, -4]
	str	A_lw, [dstin]
	str	B_lw, [dstend, -4]
	ret

	/* Copy 65..128 bytes.  */
.Lcopy128_advsimd:
	ldp	E_q, F_q, [src, 32]
	cmp	count, 96
	b.ls	.Lcopy96_advsimd
	ldp	G_q, H_q, [srcend, -64]
	stp	G_q, H_q, [dstend, -64]
.Lcopy96_advsimd:
	stp	A_q, B_q, [dstin]
	stp	E_q, F_q, [dstin, 32]
	stp	C_q, D_q, [dstend, -32]
	ret

	/* Copy 0..3 bytes using a branchless sequence.  */
.Lcopy4_advsimd:
	cbz	count, .Lcopy0_advsimd
	lsr	tmp1, count, 1
	ldrb	A_lw, [src]
	ldrb	C_lw, [srcend, -1]
	ldrb	B_lw, [src, tmp1]
	strb	A_lw, [dstin]
	strb	B_lw, [dstin, tmp1]
	strb	C_lw, [dstend, -1]
.Lcopy0_advsimd:
	ret

	.p2align 3
	/* Copy more than 128 bytes.  */
.Lcopy_long_advsimd:
	add	dstend, dstin, count

	/* Use backwards copy if there is an overlap.  */
	sub	tmp1, dstin, src
	cmp	tmp1, count
	b.lo	.Lcopy_long_backwards_advsimd

	/* Copy 16 bytes and then align src to 16-byte alignment.  */
	ldr	D_q, [src]
	and	tmp1, src, 15
	bic	src, src, 15
	sub	dst, dstin, tmp1
	add	count, count, tmp1	/* Count is now 16 too large.  */
	ldp	A_q, B_q, [src, 16]
	str	D_q, [dstin]
	ldp	C_q, D_q, [src, 48]
	subs	count, count, 128 + 16	/* Test and readjust count.  */
	b.ls	.Lcopy64_from_end_advsimd
.Lloop64_advsimd:
	stp	A_q, B_q, [dst, 16]
	ldp	A_q, B_q, [src, 80]
	stp	C_q, D_q, [dst, 48]
	ldp	C_q, D_q, [src, 112]
	add	src, src, 64
	add	dst, dst, 64
	subs	count, count, 64
	b.hi	.Lloop64_advsimd

	/* Write the last iteration and copy 64 bytes from the end.  */
.Lcopy64_from_end_advsimd:
	ldp	E_q, F_q, [srcend, -64]
	stp	A_q, B_q, [dst, 16]
	ldp	A_q, B_q, [srcend, -32]
	stp	C_q, D_q, [dst, 48]
	stp	E_q, F_q, [dstend, -64]
	stp	A_q, B_q, [dstend, -32]
	ret

	.p2align 4
	nop

	/* Large backwards copy for overlapping copies.
	   Copy 16 bytes and then align srcend to 16-byte alignment.  */
.Lcopy_long_backwards_advsimd:
	cbz	tmp1, .Lcopy0_advsimd
	ldr	D_q, [srcend, -16]
	and	tmp1, srcend, 15
	bic	srcend, srcend, 15
	sub	count, count, tmp1
	ldp	A_q, B_q, [srcend, -32]
	str	D_q, [dstend, -16]
	ldp	C_q, D_q, [srcend, -64]
	sub	dstend, dstend, tmp1
	subs	count, count, 128
	b.ls	.Lcopy64_from_start_advsimd

.Lloop64_backwards_advsimd:
	str	B_q, [dstend, -16]
	str	A_q, [dstend, -32]
	ldp	A_q, B_q, [srcend, -96]
	str	D_q, [dstend, -48]
	str	C_q, [dstend, -64]!
	ldp	C_q, D_q, [srcend, -128]
	sub	srcend, srcend, 64
	subs	count, count, 64
	b.hi	.Lloop64_backwards_advsimd

	/* Write the last iteration and copy 64 bytes from the start.  */
.Lcopy64_from_start_advsimd:
	ldp	E_q, F_q, [src, 32]
	stp	A_q, B_q, [dstend, -32]
	ldp	A_q, B_q, [src]
	stp	C_q, D_q, [dstend, -64]
	stp	E_q, F_q, [dstin, 32]
	stp	A_q, B_q, [dstin]
	ret

	.unreq	dstin
	.unreq	src
	.unreq	count
	.unreq	dst
	.unreq	srcend
	.unreq	dstend
	.unreq	A_l
	.unreq	A_lw
	.unreq	A_h
	.unreq	B_l
	.unreq	B_lw
	.unreq	B_h
	.unreq	C_lw
	.unreq	tmp1

	.unreq	A_q
	.unreq	B_q
	.unreq	C_q
	.unreq	D_q
	.unreq	E_q
	.unreq	F_q
	.unreq	G_q
	.unreq	H_q

