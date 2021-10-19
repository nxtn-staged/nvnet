// L15
c_type!(
    pub struct NDIS_OID(pub u32);
);

c_type!(
    pub enum NDIS_REQUEST_TYPE {
        NdisRequestQueryInformation = 0,
        NdisRequestSetInformation = 1,
        NdisRequestQueryStatistics = 2,
    }
);
