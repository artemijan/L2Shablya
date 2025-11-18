#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i16)]
pub enum ClanSubUnit {
    // Sub-unit types
    /** Clan subunit type of Academy */
    Academy = -1,
    /** Clan subunit type of Royal Guard A */
    Royal1 = 100,
    /** Clan subunit type of Royal Guard B */
    Royal2 = 200,
    /** Clan subunit type of Order of Knights A-1 */
    Knight1 = 1001,
    /** Clan subunit type of Order of Knights A-2 */
    Knight2 = 1002,
    /** Clan subunit type of Order of Knights B-1 */
    Knight3 = 2001,
    /** Clan subunit type of Order of Knights B-2 */
    Knight4 = 2002,
}
