// Host Java Integer.parseInt reference, not APK/Android execution.
import java.io.BufferedReader;
import java.io.InputStreamReader;
import java.nio.charset.StandardCharsets;
import java.util.Base64;
class MexParticipantIntegratorOracle {
    public static void main(String[] args) throws Exception {
        var input = new BufferedReader(new InputStreamReader(System.in, StandardCharsets.UTF_8));
        for (String line; (line = input.readLine()) != null;) {
            var value = new String(Base64.getDecoder().decode(line), StandardCharsets.UTF_8);
            try { System.out.println(Integer.toString(Integer.parseInt(value))); }
            catch (NumberFormatException ignored) { System.out.println("invalid"); }
        }
    }
}
