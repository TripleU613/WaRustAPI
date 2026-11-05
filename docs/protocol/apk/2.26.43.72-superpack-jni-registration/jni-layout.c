#include <stddef.h>
#include <stdio.h>
#include <jni.h>
int main(void) { printf("pointer_bytes=%zu method_bytes=%zu name=0x%zx signature=0x%zx fnPtr=0x%zx GetEnv=0x%zx FindClass=0x%zx RegisterNatives=0x%zx ExceptionCheck=0x%zx\n", sizeof(void *), sizeof(JNINativeMethod), offsetof(JNINativeMethod, name), offsetof(JNINativeMethod, signature), offsetof(JNINativeMethod, fnPtr), offsetof(struct JNIInvokeInterface_, GetEnv), offsetof(struct JNINativeInterface_, FindClass), offsetof(struct JNINativeInterface_, RegisterNatives), offsetof(struct JNINativeInterface_, ExceptionCheck)); return 0; }
