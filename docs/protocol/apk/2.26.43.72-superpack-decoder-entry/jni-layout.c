#include <stddef.h>
#include <stdio.h>
#include <jni.h>
int main(void) { printf("pointer_bytes=%zu SetByteArrayRegion_offset=0x%zx\n", sizeof(void *), offsetof(struct JNINativeInterface_, SetByteArrayRegion)); return 0; }
