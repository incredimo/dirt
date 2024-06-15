class Banana {
  String bananatype;

  Banana(this.bananatype) {}

  void printType() {
    print("Type should be Barangan:");
    print(bananatype);
    assert(bananatype == "Barangan");
  }
}

void main() {
  var banana = Banana("Barangan");
  banana.printType();
}
