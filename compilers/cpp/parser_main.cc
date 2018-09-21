#include "parser.h"
using namespace std;

int main() {
  for (auto expr : parse(cin)) {
    expr->display();
    cout << endl;
  }
  return 0;
}
