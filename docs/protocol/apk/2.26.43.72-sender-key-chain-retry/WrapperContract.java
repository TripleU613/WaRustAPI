package X;

// Local platform-contract fixture, not APK execution.
class EDR extends Exception {
    EDR(String message) { super(message); }
}
class EDY extends Exception {
    EDY(Throwable cause) { super(cause); }
}
public class WrapperContract {
    public static void main(String[] args) {
        String message = new EDY(new EDR("No keys for: 7")).getMessage();
        boolean barePrefix = message.startsWith("No keys for: ");
        if (barePrefix || !message.startsWith("X.EDR: ")) {
            throw new AssertionError("unexpected exception wrapper contract");
        }
        System.out.println("message=" + message);
        System.out.println("matches_bare_prefix=" + barePrefix);
    }
}
