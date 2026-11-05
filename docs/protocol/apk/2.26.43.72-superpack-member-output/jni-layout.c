#include <stddef.h>
#include <stdio.h>
#include <jni.h>
#define SHOW(x) printf(#x "=0x%zx\n", offsetof(struct JNINativeInterface_, x))
int main(void) {
    printf("pointer_bytes=%zu\n", sizeof(void *));
    SHOW(FindClass); SHOW(NewObjectArray); SHOW(NewStringUTF);
    SHOW(SetObjectArrayElement); SHOW(DeleteLocalRef);
    return 0;
}
