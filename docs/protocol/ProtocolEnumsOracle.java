// Projection of APK 2.26.37.71 BZb/BV3 enum fields and C4F/C2L verifiers.
// Minimal schema on pinned desktop protobuf-javalite; not APK execution or full message validation.
import com.google.protobuf.GeneratedMessageLite;
import com.google.protobuf.Internal;
import com.google.protobuf.InvalidProtocolBufferException;
import com.google.protobuf.Parser;
import java.io.BufferedReader;
import java.io.InputStreamReader;
import java.util.HexFormat;

public final class ProtocolEnumsOracle {
    public static final class Protocol extends GeneratedMessageLite<Protocol, Protocol.Builder> {
        private int bitField0_;
        private int type_;
        private static final Protocol DEFAULT_INSTANCE;
        private static volatile Parser<Protocol> PARSER;
        static {
            DEFAULT_INSTANCE = new Protocol();
            registerDefaultInstance(Protocol.class, DEFAULT_INSTANCE);
        }
        private Protocol() {}
        public static final class Builder extends GeneratedMessageLite.Builder<Protocol, Builder> {
            private Builder() { super(DEFAULT_INSTANCE); }
        }
        @Override
        protected Object dynamicMethod(MethodToInvoke method, Object first, Object second) {
            return switch (method) {
                case NEW_MUTABLE_INSTANCE -> new Protocol();
                case NEW_BUILDER -> new Builder();
                case BUILD_MESSAGE_INFO -> newMessageInfo(DEFAULT_INSTANCE,
                    "\u0001\u0001\u0000\u0001\u0002\u0002\u0001\u0000\u0000\u0000\u0002\u180c\u0001",
                    new Object[] {"bitField0_", "type_", (Internal.EnumVerifier) number -> number >= 0 && number <= 41 && number != 1 && number != 2 && number != 12 && number != 13 && number != 15 && number != 23 && number != 33});
                case GET_DEFAULT_INSTANCE -> DEFAULT_INSTANCE;
                case GET_PARSER -> {
                    if (PARSER == null) {
                        synchronized (Protocol.class) {
                            if (PARSER == null) PARSER = new DefaultInstanceBasedParser<>(DEFAULT_INSTANCE);
                        }
                    }
                    yield PARSER;
                }
                case GET_MEMOIZED_IS_INITIALIZED -> (byte) 1;
                case SET_MEMOIZED_IS_INITIALIZED -> null;
            };
        }
        Integer value() { return (bitField0_ & 2) == 0 ? null : type_; }
    }
    public static final class Keep extends GeneratedMessageLite<Keep, Keep.Builder> {
        private int bitField0_;
        private int type_;
        private static final Keep DEFAULT_INSTANCE;
        private static volatile Parser<Keep> PARSER;
        static {
            DEFAULT_INSTANCE = new Keep();
            registerDefaultInstance(Keep.class, DEFAULT_INSTANCE);
        }
        private Keep() {}
        public static final class Builder extends GeneratedMessageLite.Builder<Keep, Builder> {
            private Builder() { super(DEFAULT_INSTANCE); }
        }
        @Override
        protected Object dynamicMethod(MethodToInvoke method, Object first, Object second) {
            return switch (method) {
                case NEW_MUTABLE_INSTANCE -> new Keep();
                case NEW_BUILDER -> new Builder();
                case BUILD_MESSAGE_INFO -> newMessageInfo(DEFAULT_INSTANCE,
                    "\u0001\u0001\u0000\u0001\u0002\u0002\u0001\u0000\u0000\u0000\u0002\u180c\u0001",
                    new Object[] {"bitField0_", "type_", (Internal.EnumVerifier) number -> number >= 0 && number <= 2});
                case GET_DEFAULT_INSTANCE -> DEFAULT_INSTANCE;
                case GET_PARSER -> {
                    if (PARSER == null) {
                        synchronized (Keep.class) {
                            if (PARSER == null) PARSER = new DefaultInstanceBasedParser<>(DEFAULT_INSTANCE);
                        }
                    }
                    yield PARSER;
                }
                case GET_MEMOIZED_IS_INITIALIZED -> (byte) 1;
                case SET_MEMOIZED_IS_INITIALIZED -> null;
            };
        }
        Integer value() { return (bitField0_ & 2) == 0 ? null : type_; }
    }
    public static final class Envelope extends GeneratedMessageLite<Envelope, Envelope.Builder> {
        private int bitField0_;
        private Protocol protocol_;
        private Keep keep_;
        private static final Envelope DEFAULT_INSTANCE;
        private static volatile Parser<Envelope> PARSER;
        static {
            DEFAULT_INSTANCE = new Envelope();
            registerDefaultInstance(Envelope.class, DEFAULT_INSTANCE);
        }
        private Envelope() {}
        public static final class Builder extends GeneratedMessageLite.Builder<Envelope, Builder> {
            private Builder() { super(DEFAULT_INSTANCE); }
        }
        @Override
        protected Object dynamicMethod(MethodToInvoke method, Object first, Object second) {
            return switch (method) {
                case NEW_MUTABLE_INSTANCE -> new Envelope();
                case NEW_BUILDER -> new Builder();
                case BUILD_MESSAGE_INFO -> newMessageInfo(DEFAULT_INSTANCE,
                    "\u0001\u0002\u0000\u0001\u000c\u0033\u0002\u0000\u0000\u0000\u000c\u1009\u0000\u0033\u1009\u0001",
                    new Object[] {"bitField0_", "protocol_", "keep_"});
                case GET_DEFAULT_INSTANCE -> DEFAULT_INSTANCE;
                case GET_PARSER -> {
                    if (PARSER == null) {
                        synchronized (Envelope.class) {
                            if (PARSER == null) PARSER = new DefaultInstanceBasedParser<>(DEFAULT_INSTANCE);
                        }
                    }
                    yield PARSER;
                }
                case GET_MEMOIZED_IS_INITIALIZED -> (byte) 1;
                case SET_MEMOIZED_IS_INITIALIZED -> null;
            };
        }
        static Integer value(byte[] bytes, String mode) throws InvalidProtocolBufferException {
            Envelope envelope = GeneratedMessageLite.parseFrom(DEFAULT_INSTANCE, bytes);
            if (mode.equals("p")) return envelope.protocol_ == null ? null : envelope.protocol_.value();
            return envelope.keep_ == null ? null : envelope.keep_.value();
        }
    }
    public static void main(String[] args) throws Exception {
        BufferedReader input = new BufferedReader(new InputStreamReader(System.in));
        for (String line; (line = input.readLine()) != null;) {
            String[] parts = line.split("\\|", 2);
            try {
                Integer result = Envelope.value(HexFormat.of().parseHex(parts[1]), parts[0]);
                System.out.println(result == null ? "absent" : result.toString());
            } catch (InvalidProtocolBufferException | IllegalArgumentException exception) {
                System.out.println("malformed");
            }
        }
    }
}
