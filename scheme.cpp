#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <assert.h>
#include <stack>
#include <iostream>
#include <sstream>
#include"types.h"
#define ISA(v,type) (bool)(dynamic_cast<type>(v))
using namespace std;

string Null::toString(){
  return "Null";
}
bool Environment::has(string s){
  bool res = m.find(s)!=m.end() || (parent && parent->has(s));
  return res;
}
void Environment::put(string s,Expression* e){
  m[s]=e;
}
Expression* Environment::get(string s){
  return m.find(s)==m.end() ? parent->get(s) : m[s];
}
void Environment::show(){
  for(map<string,Expression*>::iterator it=m.begin();
      it!=m.end();it++)
    cout<<it->first<<" "<<it->second->toString()<<endl;
  if(parent)parent->show();
}
Expression* ExpSequence::eval(Environment* e){
  assert(es.size()>0);
  Expression* res = 0;
  for(int i=0;i<es.size();i++)
    res = es[i]->eval(e);
  return res;
}
string ExpSequence::toString(){
  string res="";
  for(int i=0;i<es.size();i++)
    res+=es[i]->toString();
  return res;
}
string BoolValue::toString(){
  return v?"#t":"#f";
}
string IntValue::toString(){
  stringstream str;
  str<<v;
  return str.str();
}
Expression* Label::eval(Environment* e){
  return e->has(s) ? e->get(s) : this;
}
string Label::toString(){
  return s;
}
Expression* Lambda::eval(Environment* e){
  Lambda* res= new Lambda(*this); 
  res->e=e;
  return res;
}
string Lambda::toString(){
  string s="(lambda (";
  for(int i=0;i<args.size();i++){
    if(i)s+=" ";
    s+=args[i];
  }
  s+=") "+body->toString()+")";
  return s;
}
string Pair::toString(){
  string res = "("+a->toString();
  Expression* t=b;
  while(ISA(t,Pair*)){
    Pair* p=(Pair*) t;
    res += " " + p->a->toString();
    t = p->b;
  }
  if(ISA(t,Null*)){
    res+=")";
  }else{
    res+=" . "+t->toString()+")";
  }
  return res;
}
string Application::toString(){
  string s="("+op->toString();
  for(int i=0;i<args.size();i++)
    s+=" "+args[i]->toString();
  s+=")";
  return s;
}
Expression* Application::eval(Environment* e){
  Expression* opVal = op->eval(e);
  vector<Expression*> vals(args.size());
  for(int i=0;i<args.size();i++)
    vals[i]=args[i]->eval(e);
  Lambda* f=0;
  if(ISA(opVal,Label*)){
    string s = ((Label*)opVal)->s;
    if(e->has(s)) f = (Lambda*)(e->get(s));
    else if(s=="cons") return new Pair(vals[0],vals[1]);
    else if(s=="car") return ((Pair*)vals[0])->a;
    else if(s=="cdr") return ((Pair*)vals[0])->b;
    else if(s=="list"){
      int i=vals.size()-1;
      if(i<0)return new Null();
      Pair* res=new Pair(vals[i--],new Null());
      while(i>=0)
        res=new Pair(vals[i--],res);
      return res;
    } else if(s=="+")return new IntValue(((IntValue*)vals[0])->v + ((IntValue*)vals[1])->v);
    else if(s=="-")return new IntValue(((IntValue*)vals[0])->v - ((IntValue*)vals[1])->v);
    else if(s=="*")return new IntValue(((IntValue*)vals[0])->v * ((IntValue*)vals[1])->v);
    else if(s=="/")return new IntValue(((IntValue*)vals[0])->v / ((IntValue*)vals[1])->v);
    else if(s=="mod")return new IntValue(((IntValue*)vals[0])->v % ((IntValue*)vals[1])->v);
    else if(s==">")return new BoolValue(((IntValue*)vals[0])->v > ((IntValue*)vals[1])->v);
    else if(s=="<")return new BoolValue(((IntValue*)vals[0])->v < ((IntValue*)vals[1])->v);
    else if(s=="and")return new BoolValue(((BoolValue*)vals[0])->v && ((BoolValue*)vals[1])->v);
    else if(s=="or")return new BoolValue(((BoolValue*)vals[0])->v || ((BoolValue*)vals[1])->v);
    else if(s=="not")return new BoolValue(!((BoolValue*)vals[0])->v);
    else if(s=="null?")return new BoolValue(ISA(vals[0],Null*));
    else if(s=="eq?" || s=="="){
      if(ISA(vals[0],BoolValue*) && ISA(vals[1],BoolValue*))
        return new BoolValue(((BoolValue*)vals[0])->v == ((BoolValue*)vals[1])->v);
      else if(ISA(vals[0],IntValue*) && ISA(vals[1],IntValue*))
        return new BoolValue(((IntValue*)vals[0])->v == ((IntValue*)vals[1])->v);
      else if(ISA(vals[0],Null*) && ISA(vals[1],Null*))
        return new BoolValue(true);
      else return new BoolValue(false);
    }
    else{
      cout<<"unknown label: "<<s<<endl;
      assert(false);
    }
  }else if(ISA(opVal,Lambda*)) f = (Lambda*)opVal;
  Environment* e2 = new Environment();
  e2->parent = f->e;
  for(int i=0;i < f->args.size();i++)
    e2->m[f->args[i]] = vals[i];
  return f->body->eval(e2);
}
string If::toString(){
  return "(if "+condition->toString()+" "+thenPart->toString()+" "+elsePart->toString()+")";
}
Expression* If::eval(Environment* e){
  BoolValue* t = (BoolValue*)(condition->eval(e));
  if(t->v)return thenPart->eval(e);
  else return elsePart->eval(e);
}
string Define::toString(){
  return "(define "+label->s+" "+exp->toString()+")"; 
}
Expression* Define::eval(Environment* e){
  e->put(label->s,exp->eval(e));
  return new Null();
}
void trim(string& s){
  int i=0;
  while(i<s.size() && s[i]==' ')
    i++;
  s=s.substr(i);
}
string parseToken(string &s){
  trim(s);
  if(s=="")return "";
  int i=0;
  if(s[i]=='(' || s[i]==')'){
    i++;
  }else{
    while(i<s.size() && s[i]!=' ' && s[i]!=')' && s[i]!='(')
      i++;
  }
  string res=s.substr(0,i);
  s=s.substr(i);
  return res;
}
bool isInteger(string s){
  int i=0;
  if(s[0]=='-')i++;
  if(!s[i])return false;
  for(;s[i];i++)
    if(s[i]<'0' || s[i]>'9')return false;
  return true;
}
Expression* parse(list<string>& tokens){
  if(tokens.front()=="("){
    tokens.pop_front();
    if(tokens.front()==")"){
      tokens.pop_front();
      return new Null();
    }if(tokens.front()=="lambda"){
      tokens.pop_front();
      Lambda* res=new Lambda();
      assert(tokens.front()=="(");
      tokens.pop_front();
      while(tokens.front()!=")"){
        res->args.push_back(tokens.front());
        tokens.pop_front();
      }
      tokens.pop_front();
      ExpSequence* exps=new ExpSequence();
      do{
        exps->es.push_back(parse(tokens));
      }while(tokens.front()!=")");
      tokens.pop_front();
      res->body = exps;
      return res;
    }else if(tokens.front()=="if"){
      tokens.pop_front();
      If *res = new If();
      res->condition = parse(tokens);
      res->thenPart = parse(tokens);
      res->elsePart = parse(tokens);
      tokens.pop_front();
      return res;
    }else if(tokens.front()=="define"){
      tokens.pop_front();
      Define *res = new Define();
      if(tokens.front()=="("){
        tokens.pop_front();
        res->label = (Label*)(parse(tokens));
        Lambda* f=new Lambda();
        while(tokens.front()!=")"){
          f->args.push_back(tokens.front());
          tokens.pop_front();
        }
        tokens.pop_front();
        ExpSequence* exps=new ExpSequence();
        do{
          exps->es.push_back(parse(tokens));
        }while(tokens.front()!=")");
        tokens.pop_front();
        f->body = exps;
        res->exp=f;
      }else{
        res->label = (Label*)(parse(tokens));
        res->exp = parse(tokens);
        tokens.pop_front();
      }
      return res;
    }else if(tokens.front()=="let"){
      tokens.pop_front();
      assert(tokens.front()=="(");
      tokens.pop_front();
      Application *res=new Application();
      Lambda* f=new Lambda();
      res->op=f;
      while(tokens.front()!=")"){
        assert(tokens.front()=="(");
        tokens.pop_front();
        f->args.push_back(((Label*)parse(tokens))->s);
        res->args.push_back(parse(tokens));
        tokens.pop_front();
      }
      tokens.pop_front();
      f->body=parse(tokens);
      tokens.pop_front();
      return res;
    }else{
      Application* res=new Application();
      res->op = parse(tokens);
      while(tokens.front()!=")"){
        res->args.push_back(parse(tokens));
      }
      tokens.pop_front();
      return res;
    }
  }else{
    string s=tokens.front();
    tokens.pop_front();
    if(s[0]=='#'){
      return new BoolValue(s[1]=='t');
    }else if(isInteger(s)){
      return new IntValue(atoi(s.c_str()));
    }
    return new Label(s);
  }
}
Expression* parse(string s){
  list<string> tokens;
  while(1){
    string t=parseToken(s);
    if(t=="")break;
    tokens.push_back(t);
  }
  ExpSequence* res=new ExpSequence();
  while(tokens.size()>0)
    res->es.push_back(parse(tokens));
  return res;
}
int main(int argc, const char *argv[])
{
  string code;
  Environment* env = new Environment();
  while(getline(cin,code)){
    Expression * e=parse(code);
    cout<<"exp: "<<e->toString()<<endl;
    string res=e->eval(env)->toString();
    cout<<"res "<<res<<endl;
  }
  return 0;
}
