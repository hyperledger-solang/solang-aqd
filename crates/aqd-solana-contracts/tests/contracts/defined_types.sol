contract DefinedTypes {
    // Enum definition
    enum Color { Red, Green, Blue }

    // Struct definition
    struct Person {
        string name;
        uint8 age;
        Color favoriteColor;
    }

    // State variables
    Person public personData;
    Color public colorData;

    @payer(payer)
    @space(1024)
	constructor(Person person) {
		personData = person;
	}

    function setColor(Color color) public {
        colorData = color;
    }

    // Getters
    function getPerson() public view returns (Person person) {
        return personData;
    }

    function getColor() public view returns (Color) {
        return colorData;
    }
}
