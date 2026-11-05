.text
.global memory_set
memory_set:
 mov x9, x0
 cbz x2, set_done
set_loop:
 strb w1, [x9], 1
 subs x2, x2, 1
 b.ne set_loop
set_done:
 ret
.global memory_copy
memory_copy:
 mov x9, x0
 cbz x2, copy_done
copy_loop:
 ldrb w10, [x1], 1
 strb w10, [x9], 1
 subs x2, x2, 1
 b.ne copy_loop
copy_done:
 ret
