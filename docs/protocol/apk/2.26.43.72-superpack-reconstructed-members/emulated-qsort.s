.text
.global emulated_qsort
emulated_qsort:
 stp x29, x30, [sp, -112]!
 stp x19, x20, [sp, 16]
 stp x21, x22, [sp, 32]
 stp x23, x24, [sp, 48]
 stp x25, x26, [sp, 64]
 stp x27, x28, [sp, 80]
 mov x19, x0
 mov x20, x1
 mov x21, x2
 mov x22, x3
 cmp x20, 2
 b.lo done
 lsr x23, x20, 1
build:
 cbz x23, sort
 sub x23, x23, 1
 mov x25, x23
 mov x24, x20
 bl sift
 b build
sort:
 sub x24, x20, 1
extract:
 cbz x24, done
 mov x27, x19
 madd x28, x24, x21, x19
 bl swap
 mov x25, 0
 bl sift
 sub x24, x24, 1
 b extract
sift:
 str x30, [sp, 96]
sift_loop:
 lsl x26, x25, 1
 add x26, x26, 1
 cmp x26, x24
 b.hs sift_done
 add x9, x26, 1
 cmp x9, x24
 b.hs chosen
 madd x27, x26, x21, x19
 madd x28, x9, x21, x19
 mov x0, x27
 mov x1, x28
 blr x22
 cmp w0, 0
 b.ge chosen
 add x26, x26, 1
chosen:
 madd x27, x25, x21, x19
 madd x28, x26, x21, x19
 mov x0, x27
 mov x1, x28
 blr x22
 cmp w0, 0
 b.ge sift_done
 bl swap
 mov x25, x26
 b sift_loop
sift_done:
 ldr x30, [sp, 96]
 ret
swap:
 mov x9, 0
swap_loop:
 cmp x9, x21
 b.hs swap_done
 ldrb w10, [x27, x9]
 ldrb w11, [x28, x9]
 strb w11, [x27, x9]
 strb w10, [x28, x9]
 add x9, x9, 1
 b swap_loop
swap_done:
 ret
done:
 ldp x19, x20, [sp, 16]
 ldp x21, x22, [sp, 32]
 ldp x23, x24, [sp, 48]
 ldp x25, x26, [sp, 64]
 ldp x27, x28, [sp, 80]
 ldp x29, x30, [sp], 112
 ret
