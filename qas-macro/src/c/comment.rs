pub struct Comment {
    pub begin: &'static str,
    pub end: &'static str,
    pub save_end: bool,
}

impl Comment {
    pub fn uncomment(code: &mut String, comments: &[Self]) {
        let mut i = 0;
        while i < code.len() {
            for comment in comments {
                if code.chars().skip(i).collect::<String>().starts_with(comment.begin) {
                    code.drain(i..i + comment.begin.len());
                    while !code[i..].starts_with(comment.end) {
                        code.remove(i);
                    }
                    if !comment.save_end {
                        code.drain(i..i + comment.end.len());
                    }
                }
            }
            i += 1
        }
    }
}
