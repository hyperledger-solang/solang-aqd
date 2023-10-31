contract unsigned_int {
    uint8 public uint8Value;
    uint16 public uint16Value;
    uint32 public uint32Value;
    uint64 public uint64Value;
    uint128 public uint128Value;
    uint256 public uint256Value;

    @payer(payer)
    constructor(
        uint8 initUint8Value,
        uint16 initUint16Value,
        uint32 initUint32Value,
        uint64 initUint64Value,
        uint128 initUint128Value,
        uint256 initUint256Value
    ) {
        uint8Value = initUint8Value;
        uint16Value = initUint16Value;
        uint32Value = initUint32Value;
        uint64Value = initUint64Value;
        uint128Value = initUint128Value;
        uint256Value = initUint256Value;
    }

    function getUint8Value() public view returns (uint8) {
        return uint8Value;
    }

    function getUint16Value() public view returns (uint16) {
        return uint16Value;
    }

    function getUint32Value() public view returns (uint32) {
        return uint32Value;
    }

    function getUint64Value() public view returns (uint64) {
        return uint64Value;
    }

    function getUint128Value() public view returns (uint128) {
        return uint128Value;
    }

    function getUint256Value() public view returns (uint256) {
        return uint256Value;
    }
}
