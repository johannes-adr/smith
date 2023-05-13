pub fn read_into<const CAP: usize, T>(
    buff: &mut [T; CAP],
    iter: &mut impl Iterator<Item = T>,
) -> usize {
    for i in 0..CAP {
        if let Some(t) = iter.next() {
            buff[i] = t
        } else {
            return i;
        }
    }
    return CAP;
}
