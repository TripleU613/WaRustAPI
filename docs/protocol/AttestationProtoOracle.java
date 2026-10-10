// APK 2.26.37.71 BP2.dynamicMethod descriptor and field-array reconstruction.
// This runs on a pinned desktop protobuf-javalite runtime, not an Android APK.
// No account, key, signature, network or file state is accessed.
import com.google.protobuf.GeneratedMessageLite;
import com.google.protobuf.Internal;
import com.google.protobuf.InvalidProtocolBufferException;
import com.google.protobuf.Parser;
import java.io.BufferedReader;
import java.io.InputStreamReader;
import java.util.HexFormat;

public final class AttestationProtoOracle {
    public static final class Body extends GeneratedMessageLite<Body, Body.Builder> {
        private int bitField0_;
        private int accountType_;
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
                    "\u0001\u0001\u0000\u0001\u0001\u0001\u0001\u0000\u0000\u0000\u0001\u180c\u0000",
                    new Object[] {"bitField0_", "accountType_",
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
        public static int parseType(byte[] body) throws InvalidProtocolBufferException {
            return GeneratedMessageLite.parseFrom(DEFAULT_INSTANCE, body).accountType_;
        }
    }
    public static void main(String[] args) throws Exception {
        BufferedReader input = new BufferedReader(new InputStreamReader(System.in));
        for (String line; (line = input.readLine()) != null;) {
            try {
                System.out.println(Body.parseType(HexFormat.of().parseHex(line)));
            } catch (InvalidProtocolBufferException | IllegalArgumentException exception) {
                System.out.println("malformed");
            }
        }
    }
}
