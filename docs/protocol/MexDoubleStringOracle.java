// Executed host Java Double and pinned AOSP container formatting reference.
import java.io.BufferedReader;
import java.io.InputStreamReader;
import java.nio.charset.StandardCharsets;
import java.util.Base64;
import org.json.JSONObject;
class MexDoubleStringOracle {
    public static void main(String[] args) throws Exception {
        var input = new BufferedReader(new InputStreamReader(System.in, StandardCharsets.UTF_8));
        for (String line; (line = input.readLine()) != null;) {
            double value = Double.longBitsToDouble(Long.parseUnsignedLong(line, 16));
            String container;
            try { container = JSONObject.numberToString(value); }
            catch (org.json.JSONException ignored) { container = "invalid"; }
            System.out.println(Double.toString(value) + "\t" + container);
        }
    }
}
