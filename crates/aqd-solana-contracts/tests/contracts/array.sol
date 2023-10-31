contract array {
    uint8[4] public intArray;

    @payer(payer)
    constructor(
        uint8[4] initIntArray
    ) {
        intArray = initIntArray;
    }

    function getIntArray() public view returns (uint8[4]) {
        return intArray;
    }
}
