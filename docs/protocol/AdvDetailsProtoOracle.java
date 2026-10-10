// APK 2.26.37.71 GsB.dynamicMethod descriptor and field-array reconstruction.
// This runs on a pinned desktop protobuf-javalite runtime, not an Android APK.
// No account, key, signature, network or file state is accessed.
import com.google.protobuf.GeneratedMessageLite;
import com.google.protobuf.Internal;
import com.google.protobuf.InvalidProtocolBufferException;
import com.google.protobuf.Parser;
import java.io.BufferedReader;
import java.io.InputStreamReader;
import java.util.HexFormat;

public final class AdvDetailsProtoOracle {
    public static final class Body extends GeneratedMessageLite<Body, Body.Builder> {
        private int bitField0_;
        private int rawId_;
        private long timestamp_;
        private int keyIndex_;
        private int accountType_;
        private int deviceType_;
        private static final Body DEFAULT_INSTANCE;
        private static volatile Parser<Body> PARSER;
        static {
            DEFAULT_INSTANCE = new Body();
            registerDefaultInstance(Body.class, DEFAULT_INSTANCE);
        }
        private Body() {}
        public static final class Builder extends GeneratedMessageLite.Builder<Body, Builder> {
            private Builder() { super(DEFAULT_INSTANCE); }
        }
        @Override
        protected Object dynamicMethod(MethodToInvoke method, Object first, Object second) {
            return switch (method) {
                case NEW_MUTABLE_INSTANCE -> new Body();
                case NEW_BUILDER -> new Builder();
                case BUILD_MESSAGE_INFO -> newMessageInfo(DEFAULT_INSTANCE,
                    "\u0001\u0005\u0000\u0001\u0001\u0005\u0005\u0000\u0000\u0000\u0001\u100b\u0000\u0002\u1003\u0001\u0003\u100b\u0002\u0004\u180c\u0003\u0005\u180c\u0004",
                    new Object[] {"bitField0_", "rawId_", "timestamp_", "keyIndex_", "accountType_",
                        (Internal.EnumVerifier) number -> number >= 0 && number <= 2, "deviceType_",
                        (Internal.EnumVerifier) number -> number >= 0 && number <= 2});
                case GET_DEFAULT_INSTANCE -> DEFAULT_INSTANCE;
                case GET_PARSER -> {
                    if (PARSER == null) {
                        synchronized (Body.class) {
                            if (PARSER == null) PARSER = new DefaultInstanceBasedParser<>(DEFAULT_INSTANCE);
                        }
                    }
                    yield PARSER;
                }
                case GET_MEMOIZED_IS_INITIALIZED -> (byte) 1;
                case SET_MEMOIZED_IS_INITIALIZED -> null;
            };
        }
        public static String project(byte[] body) throws InvalidProtocolBufferException {
            Body parsed = GeneratedMessageLite.parseFrom(DEFAULT_INSTANCE, body);
            return HexFormat.of().formatHex(parsed.toByteArray()) + "|" +
                ((parsed.bitField0_ & 1) != 0 ? Integer.toUnsignedString(parsed.rawId_) : "null") + "|" +
                ((parsed.bitField0_ & 2) != 0 ? Long.toUnsignedString(parsed.timestamp_) : "null") + "|" +
                ((parsed.bitField0_ & 4) != 0 ? Integer.toUnsignedString(parsed.keyIndex_) : "null") + "|" +
                ((parsed.bitField0_ & 8) != 0 ? Integer.toString(parsed.accountType_) : "null") + "|" +
                ((parsed.bitField0_ & 16) != 0 ? Integer.toString(parsed.deviceType_) : "null");
        }
    }
    public static void main(String[] args) throws Exception {
        BufferedReader input = new BufferedReader(new InputStreamReader(System.in));
        for (String line; (line = input.readLine()) != null;) {
            try {
                System.out.println(Body.project(HexFormat.of().parseHex(line)));
            } catch (InvalidProtocolBufferException | IllegalArgumentException exception) {
                System.out.println("malformed");
            }
        }
    }
}
