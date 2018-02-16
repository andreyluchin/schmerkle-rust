pub trait Digest {
    fn input(&mut self, input: &[u8]);
    fn result(&mut self, output: &[u8]);
    fn reset(&mut self);
}

pub trait DigestHash {
    fn hash<D: Digest>(&self, state: &mut D);
}