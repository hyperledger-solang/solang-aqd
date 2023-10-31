contract signed_int {
    int8 public i8Value;
    int16 public i16Value;
    int32 public i32Value;
    int64 public i64Value;
    int128 public i128Value;
    int256 public i256Value;

    @payer(payer)
    constructor(
        int8 initi8Value,
        int16 initi16Value,
        int32 initi32Value,
        int64 initi64Value,
        int128 initi128Value,
        int256 initi256Value
    ) {
        i8Value = initi8Value;
        i16Value = initi16Value;
        i32Value = initi32Value;
        i64Value = initi64Value;
        i128Value = initi128Value;
        i256Value = initi256Value;
    }

    function getI8Value() public view returns (int8) {
        return i8Value;
    }

    function getI16Value() public view returns (int16) {
        return i16Value;
    }

    function getI32Value() public view returns (int32) {
        return i32Value;
    }

    function getI64Value() public view returns (int64) {
        return i64Value;
    }

    function getI128Value() public view returns (int128) {
        return i128Value;
    }

    function getI256Value() public view returns (int256) {
        return i256Value;
    }
}
