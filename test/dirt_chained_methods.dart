class Christmas {
  Christmas() {}

  void christmas() {
    print("I should print twice.");
  }
}

class Before {
  var c = new Christmas();
  Before() {}

  before() {
    return c;
  }
}

class Anything {
  var b = Before();
  Anything() {}

  anything() {
    return b;
  }
}

void main() {
  var ob = Anything();
  ob.anything().before().christmas();
  Anything().anything().before().christmas();
}
