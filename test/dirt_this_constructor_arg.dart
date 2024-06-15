class Fox {
  String say;

  Fox(this.say) {}

  void whatDoesTheFoxSay() {
    print(say);
  }
}

void main() {
  print("What the fox says:");
  var fox = Fox("ringdingding");
  fox.whatDoesTheFoxSay();
}
