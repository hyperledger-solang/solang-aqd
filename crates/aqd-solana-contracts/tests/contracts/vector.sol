contract VectorTest {
    uint8[] public intArray;

    @payer(payer)
    @space(1024)
    constructor(uint8[] memory initIntArray) {
        intArray = initIntArray;
    }

    function getIntArray() public view returns (uint8[] memory) {
        return intArray;
    }
}
