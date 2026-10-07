/* Fail-closed asset imports for the local file-backed decoder probe. */
extern int puts(const char *);
extern void abort(void);
static void unsupported(const char *name) {
  puts(name);
  abort();
}
void AAsset_close(void *asset) {
  unsupported("unsupported asset import: AAsset_close");
}
void *AAssetManager_fromJava(void *env, void *manager) {
  unsupported("unsupported asset import: AAssetManager_fromJava");
  return 0;
}
void *AAssetManager_open(void *manager, const char *name, int mode) {
  unsupported("unsupported asset import: AAssetManager_open");
  return 0;
}
int AAsset_openFileDescriptor(void *asset, void *start, void *length) {
  unsupported("unsupported asset import: AAsset_openFileDescriptor");
  return -1;
}
