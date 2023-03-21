use anyhow::Context;
use chardetng::EncodingDetector;
use encoding_rs::Encoding;

pub enum Coding<'a> {
    UTF8 {
        output: String,
        #[allow(unused)]
        src_coding: &'static Encoding,
    },
    Source(&'a [u8]),
}

impl AsRef<[u8]> for Coding<'_> {
    fn as_ref(&self) -> &[u8] {
        match self {
            Coding::UTF8 { output, .. } => output.as_bytes(),
            Coding::Source(src) => src,
        }
    }
}

impl<'a> Coding<'a> {
    pub fn new(source: &'a [u8]) -> Coding {
        Coding::Source(source)
    }

    pub async fn parse(self) -> anyhow::Result<Coding<'a>> {
        struct LocalPtrWrap(*const u8, usize);
        unsafe impl Send for LocalPtrWrap {}
        let source = match self {
            Coding::Source(source) => source,
            _ => {
                panic!("bad state")
            }
        };
        let local_ptr = LocalPtrWrap(source.as_ptr(), source.len());
        tokio::task::spawn_blocking(move || {
            let _ = &local_ptr;
            let src = unsafe {
                &*std::ptr::slice_from_raw_parts(
                    &*local_ptr.0, local_ptr.1)
            };
            let mut enc_doctor = EncodingDetector::new();
            enc_doctor.feed(src, true);
            let coding = enc_doctor.guess(None, true);
            // safety: src is utf8 well-formed
            if coding == encoding_rs::UTF_8 {
                return Coding::Source(src);
            }
            let mut d = coding.new_decoder();
            let buf_len = d.max_utf8_buffer_length(
                src.len()).unwrap_or(0);
            let mut out_s = String::with_capacity(buf_len);
            let _ = d.decode_to_string(src, &mut out_s, true);
            Coding::UTF8 {
                output: out_s,
                src_coding: coding,
            }
        })
            .await
            .context("unexpected err")
    }
}
