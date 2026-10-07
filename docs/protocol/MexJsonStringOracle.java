// Independent pinned-AOSP org.json execution, not APK/Android execution.
import java.io.BufferedReader;
import java.io.InputStreamReader;
import java.nio.charset.StandardCharsets;
import java.util.Base64;
import org.json.JSONObject;
class MexJsonStringOracle {
    public static void main(String[] args) throws Exception {
        var input = new BufferedReader(new InputStreamReader(System.in, StandardCharsets.UTF_8));
        for (String line; (line = input.readLine()) != null;) {
            var raw = new String(Base64.getDecoder().decode(line), StandardCharsets.UTF_8);
            var model = new JSONObject(raw);
            var output = new JSONObject();
            output.put("default_string", model.optString("value"));
            output.put("nullable_string", model.isNull("value") ? JSONObject.NULL : model.optString("value"));
            output.put("container", model.toString());
            System.out.println(output.toString());
        }
    }
}
