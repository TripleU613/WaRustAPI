// Host-reference compilation only; no runtime JSON behavior.
package android.annotation; @java.lang.annotation.Target({java.lang.annotation.ElementType.TYPE_USE, java.lang.annotation.ElementType.CONSTRUCTOR, java.lang.annotation.ElementType.METHOD, java.lang.annotation.ElementType.FIELD}) public @interface SystemApi { enum Client { MODULE_LIBRARIES } Client client(); }
