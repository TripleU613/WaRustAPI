// Independent host Java UTF-16 String.hashCode reference; not APK execution.
import java.io.BufferedReader;
import java.io.InputStreamReader;
import java.nio.charset.StandardCharsets;
import java.util.Base64;
class MexParticipantHashOracle {
    public static void main(String[] args) throws Exception {
        var input = new BufferedReader(new InputStreamReader(System.in, StandardCharsets.UTF_8));
        for (String line; (line = input.readLine()) != null;) {
            var value = new String(Base64.getDecoder().decode(line), StandardCharsets.UTF_8);
            System.out.println(Integer.toUnsignedString(value.hashCode(), 16));
        }
    }
}
