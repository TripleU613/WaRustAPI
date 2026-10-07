/* Local ARM64 artifact probe, not a Java VM or WhatsApp client.
 * JNI table indexes: Oracle JNI Functions/Invocation API; ARM64 registrations
 * and range signature: APK 2.26.37.71 libsuperpack.so, 0x96d0 and 0x9a50.
 * All unsupported callbacks abort. Strings are ASCII fixture paths only.
 */
typedef unsigned long size_t;
extern void __libc_init(void *, void *, int (*)(int, char **, char **),
                        const void *);
extern int puts(const char *);
extern void *dlopen(const char *, int);
extern void *dlsym(void *, const char *);
extern char *dlerror(void);
extern int strcmp(const char *, const char *);
extern int printf(const char *, ...);
extern void abort(void);
extern void *calloc(size_t, size_t);
extern char *strdup(const char *);
extern size_t strlen(const char *);
struct NativeMethod {
  const char *name;
  const char *sig;
  void *fn;
};
static void *env_table[234], *jni_env = env_table;
static void *vm_table[8], *jni_vm = vm_table;
static int registered, exception_pending;
static void *range_entry;
struct StringArray {
  int length;
  char **items;
};
static void unsupported(void) {
  puts("unsupported JNI callback");
  abort();
}
static void *find_class(void *env, const char *name) {
  if (strcmp(name, "com/facebook/superpack/AssetDecompressor") &&
      strcmp(name, "com/whatsapp/superpack/WhatsAppObiInputStream") &&
      strcmp(name, "com/whatsapp/superpack/WhatsAppOpenboxArchive") &&
      strcmp(name, "java/lang/String") &&
      strcmp(name, "com/facebook/superpack/AssetDecompressionException")) {
    puts(name);
    unsupported();
  }
  return (void *)name;
}
static unsigned char exception_check(void *env) {
  return exception_pending != 0;
}
static int throw_new(void *env, void *klass, const char *message) {
  printf("native exception class=%s reason=%s\n", (char *)klass, message);
  exception_pending = 1;
  return 0;
}
static void ascii(const char *s) {
  if (!s || strlen(s) > 4096)
    unsupported();
  for (const unsigned char *p = (const unsigned char *)s; *p; p++)
    if (*p >= 128)
      unsupported();
}
static char *get_utf(void *env, char *s, unsigned char *copy) {
  ascii(s);
  if (copy)
    *copy = 0;
  return s;
}
static void release_utf(void *env, char *s, const char *chars) {
  if (s != chars)
    unsupported();
}
static char *new_utf(void *env, const char *s) {
  ascii(s);
  char *r = strdup(s);
  if (!r)
    unsupported();
  return r;
}
static void
delete_ref(void *env,
           void *ref) { /* Objects live until this isolated probe exits. */ }
static struct StringArray *new_array(void *env, int length, void *klass,
                                     void *init) {
  if (length < 0 || length > 1024 || init || strcmp(klass, "java/lang/String"))
    unsupported();
  struct StringArray *a = calloc(1, sizeof(*a));
  if (!a)
    unsupported();
  a->length = length;
  a->items = calloc(length ? length : 1, sizeof(char *));
  if (!a->items)
    unsupported();
  return a;
}
static void set_array(void *env, struct StringArray *array, int index,
                      char *value) {
  if (!array || index < 0 || index >= array->length)
    unsupported();
  ascii(value);
  array->items[index] = value;
}
static int register_natives(void *env, void *klass,
                            const struct NativeMethod *methods, int count) {
  printf("register class=%s count=%d\n", (char *)klass, count);
  if (count < 0 || count > 32)
    unsupported();
  for (int i = 0; i < count; i++) {
    if (!methods[i].name || !methods[i].sig || !methods[i].fn)
      unsupported();
    printf("native=%s signature=%s\n", methods[i].name, methods[i].sig);
  }
  for (int i = 0; i < count; i++)
    if (!strcmp(methods[i].name, "decompress_range_from_so")) {
      if (strcmp(methods[i].sig, "(Ljava/lang/String;Ljava/lang/String;Ljava/"
                                 "lang/String;II)[Ljava/lang/String;") ||
          range_entry)
        unsupported();
      range_entry = methods[i].fn;
    }
  registered += count;
  return 0;
}
static int get_env(void *vm, void **out, int version) {
  if (version != 0x10006)
    return -3;
  *out = &jni_env;
  return 0;
}
static void (*ctors[])(void) = {(void (*)(void))-1, 0};
int main(int argc, char **argv, char **envp) {
  void *decoder = dlopen("/system/lib64/libsuperpack.so", 2);
  if (!decoder) {
    puts(dlerror());
    return 2;
  }
  void *entry = dlsym(decoder, "JNI_OnLoad");
  if (!entry) {
    puts(dlerror());
    return 3;
  }
  puts("decoder loaded; JNI_OnLoad resolved");
  for (int i = 4; i < 234; i++)
    env_table[i] = (void *)unsupported;
  for (int i = 3; i < 8; i++)
    vm_table[i] = (void *)unsupported;
  /* JNI function table slots, not APK protobuf fields. */
  env_table[6] = (void *)find_class;
  env_table[14] = (void *)throw_new;
  env_table[23] = (void *)delete_ref;
  env_table[167] = (void *)new_utf;
  env_table[169] = (void *)get_utf;
  env_table[170] = (void *)release_utf;
  env_table[172] = (void *)new_array;
  env_table[174] = (void *)set_array;
  env_table[215] = (void *)register_natives;
  env_table[228] = (void *)exception_check;
  vm_table[6] = (void *)get_env;
  int result = ((int (*)(void *, void *))entry)(&jni_vm, 0);
  printf("JNI_OnLoad result=0x%x registered=%d\n", result, registered);
  if (result != 0x10006 || registered != 19 || exception_pending)
    return 4;
  if (argc == 2 && !strcmp(argv[1], "--first-entry")) {
    /* APK entry requests flags=6 (NOW|NOLOAD); preload this local ELF. */
    void *archive = dlopen("/system/lib64/wa_archive.so", 2);
    if (!archive) {
      puts(dlerror());
      return 5;
    }
    if (!range_entry)
      return 6;
    /* 0xa964 loops while index < end: [0,1) selects one output entry. */
    struct StringArray *files =
        ((struct StringArray * (*)(void *, void *, const char *, const char *,
                                   const char *, int, int))
             range_entry)(&jni_env, 0, "/system/lib64/wa_archive.so", "spo",
                          "/decoded", 0, 1);
    if (exception_pending || !files)
      return 7;
    printf("returned files=%d\n", files->length);
    for (int i = 0; i < files->length; i++)
      puts(files->items[i]);
  } else if (argc != 1)
    return 8;
  return 0;
}
void start_c(void *raw) {
  const void *structors[] = {ctors, ctors, ctors};
  __libc_init(raw, 0, main, structors);
}
__asm__(".global _start\n_start:\nmov x0, sp\nb start_c\n");
