// Independent JVM numeric oracle for selected APK43.72 0pD.A01 arithmetic.
// This executes Java parsing/arithmetic only, never the APK or a network request.
class ContactRefusalOracle {
    static String quote(String value) {
        return value == null ? "null" : "\"" + value.replace("\\", "\\\\").replace("\"", "\\\"") + "\"";
    }
    public static void main(String[] args) {
        String[][] cases = {
            {null,null},{"503",null},{null,"30"},{"503","30"},{"+503","+30"},
            {"٥٠٣","٣٠"},{"0","0"},{"-1","-1"},{"503","7200"},
            {"503","3600"},{"503","3601"},{"503","-7200"},
            {"2147483647","1"},{"-2147483648","1"},
            {"2147483648","30"},{"-2147483649","30"},{"","30"},{" 503","30"},
            {"503",""},{"503"," 30"},{"503","9223372036854775808"},
            {"503","9223372036854775807"},{"503","-9223372036854775808"},
            {"503","9223372036854776"},{"503","--1"},{"bad","bad"}
        };
        StringBuilder out = new StringBuilder("{\n  \"cases\": [\n");
        for (int i=0;i<cases.length;i++) {
            String[] input=cases[i]; Integer code=null; Long backoff=null; String failure=null;
            try { backoff=Math.min((input[1]==null ? 7200L : Long.parseLong(input[1]))*1000L,3600000L); }
            catch (NumberFormatException e) { failure="backoff"; }
            if (failure==null) {
                try { code=input[0]==null ? -1 : Integer.parseInt(input[0]); }
                catch (NumberFormatException e) { failure="code"; }
            }
            if(i>0) out.append(",\n");
            out.append("    {\"code_text\": ").append(quote(input[0]))
                .append(", \"backoff_text\": ").append(quote(input[1]))
                .append(", \"code\": ").append(code)
                .append(", \"backoff_millis\": ").append(backoff)
                .append(", \"failure\": ").append(quote(failure)).append("}");
        }
        System.out.println(out.append("\n  ]\n}"));
    }
}
