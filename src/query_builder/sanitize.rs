pub fn sanitize_by_quote(input: &str, output: &mut String) {
    output.push('"');

    let mut s = input.chars();
    while let Some(next) = s.next() {
        match next {
            '"' => {
                output.push(next);
                output.push('"');
            }
            '\'' => {
                output.push(next);
                output.push('\'');
            }
            '\\' => {
                output.push(next);
                output.push('\\');
            }
            n => output.push(n),
        }
    }

    output.push('"');
}
