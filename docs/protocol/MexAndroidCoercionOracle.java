// Independent host execution of pinned AOSP org.json, not APK/Android execution.
// Compile with the six source files/hash manifest cited in mex.md; annotation
// stubs affect compilation only and do not replace any JSON conversion behavior.
import java.io.BufferedReader;
import java.io.InputStreamReader;
import java.nio.charset.StandardCharsets;
import java.util.Base64;
import org.json.JSONObject;

class MexAndroidCoercionOracle {
    public static void main(String[] args) throws Exception {
        BufferedReader input = new BufferedReader(new InputStreamReader(System.in, StandardCharsets.UTF_8));
        for (String line; (line = input.readLine()) != null;) {
            String text = new String(Base64.getDecoder().decode(line), StandardCharsets.UTF_8);
            JSONObject model = new JSONObject(text);
            int integer = model.optInt("value");
            boolean value = model.optBoolean("value", integer == 1);
            JSONObject output = new JSONObject();
            output.put("integer", Integer.toString(integer));
            output.put("boolean", value);
            output.put("nullable_boolean", model.isNull("value") ? JSONObject.NULL : value);
            output.put("join_time", integer > 0 ? Long.toString((long)integer) : JSONObject.NULL);
            Object source = model.opt("value");
            Object bits = JSONObject.NULL;
            if (source instanceof String) {
                try { bits = Long.toUnsignedString(Double.doubleToRawLongBits(Double.parseDouble((String)source)), 16); }
                catch (NumberFormatException ignored) { }
            }
            output.put("string_double_bits", bits);
            System.out.println(output.toString());
        }
    }
}
