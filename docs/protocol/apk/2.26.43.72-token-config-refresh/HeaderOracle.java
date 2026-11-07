import java.nio.ByteBuffer;
import java.nio.ByteOrder;

// Analyst transcription of MGo.A00's header branch, not APK execution.
// Uses synthetic bytes and Java ByteBuffer signed-byte/cursor semantics.
class HeaderOracle {
    static String check(ByteBuffer b, int size) {
        if (size < 8) return "small";
        if (b.limit() < size) return "limit";
        if (b.get() == (byte) 0x3c && b.get() == (byte) 0xf1) {
            byte version = b.get();
            if (version <= 0 || version > 1) return "version";
            byte offset = b.get();
            if (offset <= 0 || offset >= size - 2) return "offset";
            if (b.getInt() != size) return "size";
            b.position(size - 2);
            byte one = b.get(), two = b.get();
            if (one != (byte) 0x3c || two != (byte) 0xf1) return "footer";
            b.position(offset);
            return "ok";
        }
        b.position(0);
        return "raw";
    }
    static ByteBuffer frame(int size) {
        ByteBuffer b = ByteBuffer.allocate(size).order(ByteOrder.LITTLE_ENDIAN);
        b.put((byte)0x3c).put((byte)0xf1).put((byte)1).put((byte)8).putInt(size);
        b.put(size-2, (byte)0x3c).put(size-1, (byte)0xf1);
        b.position(0);
        return b;
    }
    public static void main(String[] args) {
        System.out.println("[");
        for (int i = 0; i < 12; i++) {
            ByteBuffer b = frame(32);
            int size = 32;
            String name;
            switch (i) {
                case 0: name="valid_frame"; break;
                case 1: name="raw_no_magic"; b.put(0,(byte)0); break;
                case 2: name="partial_magic"; b.put(1,(byte)0); break;
                case 3: name="small_size"; size=7; break;
                case 4: name="short_limit"; b.limit(31); break;
                case 5: name="zero_version"; b.put(2,(byte)0); break;
                case 6: name="future_version"; b.put(2,(byte)2); break;
                case 7: name="zero_offset"; b.put(3,(byte)0); break;
                case 8: name="signed_negative_offset"; b.put(3,(byte)128); break;
                case 9: name="footer_offset"; b.put(3,(byte)30); break;
                case 10: name="size_mismatch"; b.putInt(4,31); break;
                default: name="bad_footer"; b.put(31,(byte)0); break;
            }
            String result = check(b,size);
            System.out.printf("  {\"case\":\"%s\",\"result\":\"%s\",\"position\":%d}%s%n",name,result,b.position(),i==11?"":",");
        }
        System.out.println("]");
    }
}
