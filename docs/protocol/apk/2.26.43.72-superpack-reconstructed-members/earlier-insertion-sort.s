.text
.global emulated_qsort
emulated_qsort:
 stp x29, x30, [sp, -80]!
 stp x19, x20, [sp, 16]
 stp x21, x22, [sp, 32]
 stp x23, x24, [sp, 48]
 stp x25, x26, [sp, 64]
 mov x19, x0
 mov x20, x1
 mov x21, x2
 mov x22, x3
 mov x23, 1
outer:
 cmp x23, x20
 b.hs done
 mov x24, x23
inner:
 cbz x24, next
 madd x26, x24, x21, x19
 sub x25, x26, x21
 mov x0, x25
 mov x1, x26
 blr x22
 cmp w0, 0
 b.le next
 mov x9, 0
swap:
 cmp x9, x21
 b.hs swapped
 ldrb w10, [x25, x9]
 ldrb w11, [x26, x9]
 strb w11, [x25, x9]
 strb w10, [x26, x9]
 add x9, x9, 1
 b swap
swapped:
 sub x24, x24, 1
 b inner
next:
 add x23, x23, 1
 b outer
done:
 ldp x19, x20, [sp, 16]
 ldp x21, x22, [sp, 32]
 ldp x23, x24, [sp, 48]
 ldp x25, x26, [sp, 64]
 ldp x29, x30, [sp], 80
 ret
