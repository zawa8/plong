cplong has dedicated digits for last siks heksadesiml dizits:
1. L as ten
2. Y xz yilewen
3. V xz twelw
4. W xz dblun=8+5
5. P xz purxn=8+6
6. F xz fiwxn=8+7

so  8+8=10=4*4=F+1=P+2=W+3=V+4=Y+5=L+6.

```
class cplong{ // zust c++ pseudo code

public :
u8 wlyu[];
s8 start_prisizxn_leyr;
bool is_negetiw;

public static:
s8 _meksimun_saiz_of_wlyu_array_ ;

}

s8 main() {

cplong._meksimun_saiz_of_wlyu_array_ = 8 ;

cplong a([5V],1,0)//same as float a= 5.75
// a.wlyu is [5V] , a.start_prisizxn_leyr is 1
// a.is_negetiw is 0 so a is pozitiw

cplong a2([5,V],0,0)//same as float a= 5.75
// a2.wlyu is u8 array  [5,V]
// a2.start_prisizxn_leyr is 0
// for nekst u8 number V  prisizxn_leyr is 0+1
// a2.is_negetiw is 0 so a2 is pozitiw
// a2 is 5 + V/(F+1)



cplong a1([4,4,0,4],0,1);// cplong a1("4.4.0.4",0,1)
// à1.wlyu is 4+4/10+4/1000
// à1.start_prisizxn_leyr is 0
// à1 prisizxn_leyrs are 0,1,2,3
// a1.is_negetiw is 1 so a1 is negetiw

cplong b([6],0,0)//same as float b = 6
// a.wlyu is [6] , b.start_prisizxn_leyr is 0
// b.is_negetiw is 0 so b is pozitiw

cplong d = a - b;
// d wil bi = ("4",1,1)

}
```

