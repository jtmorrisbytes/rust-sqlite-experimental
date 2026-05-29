use super::Sqlite3Memory;

impl PartialEq for Sqlite3Memory {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            use core::arch::aarch64::*;

            // Compare first 16 bytes
            let cmp0 = vceqq_u8(
                transmute(self.0[0]),
                transmute(other.0[0]),
            );

            // Compare second 16 bytes
            let cmp1 = vceqq_u8(
                transmute(self.0[1]),
                transmute(other.0[1]),
            );

            // Reduce both vectors to a single "all equal" test
            let min0 = vminvq_u8(cmp0);
            let min1 = vminvq_u8(cmp1);

            (min0 == 0xFF) & (min1 == 0xFF)
        }
    }
}

impl Eq for Sqlite3Memory {}