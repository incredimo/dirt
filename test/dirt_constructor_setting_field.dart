class Banana {
  String bananatype;

  Banana(this.bananatype) {
    bananatype = "Pisang";
  }

  void printType() {
    print("Type should be Pisang:");
    print(bananatype);
    assert(bananatype == "Pisang");
  }
}

void main() {
  var banana = Banana("Pisang");
  banana.printType();
}
