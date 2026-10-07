// Independent reference for APK 37.71 0Gv.A01 and 8kY.A0B calls.
// Executes host Java library semantics; it does not execute Android or the APK.
import java.io.BufferedReader;
import java.io.InputStreamReader;
import java.nio.charset.StandardCharsets;
import java.util.Base64;
import java.util.Locale;

class MexJavaNumericOracle {
    // Names and ordinals transcribed from APK 37.71 2hu.<clinit>.
    enum Role { UNSET_OR_UNRECOGNIZED_ENUM_VALUE, ADMIN_MEMBER, MEMBER, SUPERADMIN_MEMBER }
    public static void main(String[] args) throws Exception {
        if (args.length == 1 && args[0].equals("digits")) {
            for (int code = 0; code <= 0xffff; code++) {
                int digit = Character.digit((char) code, 10);
                if (digit >= 0) System.out.println(code + "\t" + digit);
            }
            return;
        }
        BufferedReader input = new BufferedReader(new InputStreamReader(System.in, StandardCharsets.UTF_8));
        for (String line; (line = input.readLine()) != null;) {
            String value = new String(Base64.getDecoder().decode(line), StandardCharsets.UTF_8);
            if (args.length == 1 && args[0].equals("roles")) {
                Role role;
                try { role = Role.valueOf(value.toUpperCase(Locale.US)); }
                catch (IllegalArgumentException ignored) { role = Role.UNSET_OR_UNRECOGNIZED_ENUM_VALUE; }
                String memberRole = switch (role.ordinal()) {
                    case 1 -> "admin";
                    case 3 -> "superadmin";
                    default -> "";
                };
                System.out.println(role.name() + "\t" + memberRole);
                continue;
            }
            long parsed;
            try { parsed = Long.parseLong(value); }
            catch (NumberFormatException ignored) { parsed = 0; }
            System.out.println(Long.toString(parsed) + "\t" + Long.toString(parsed * 1000L));
        }
    }
}
