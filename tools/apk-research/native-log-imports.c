/* Render local decoder diagnostics; no Android logging service is emulated. */
extern int puts(const char *);
extern int vprintf(const char *, __builtin_va_list);
int __android_log_print(int priority, const char *tag, const char *format,
                        ...) {
  puts(tag);
  __builtin_va_list args;
  __builtin_va_start(args, format);
  int n = vprintf(format, args);
  __builtin_va_end(args);
  puts("");
  return n;
}
