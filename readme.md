# Mud

Visualize data from a csv file. Missing or invalid values are replaced with zero (-1.0).

```bash
id,price,quantity
1,20,7
2,15,3
3,16.50,4
```

```bash
mud data.csv sort price;

*------------------------------*
* id    * price    * amount    *
*-------*----------*-----------*
* 2    * 15.00    * 3          *
* 3    * 16.50    * 4          *
* 1    * 20       * 7          *
*------------------------------*

mud data.csv filter price gt 15.0;

*------------------------------*
* id    * price    * amount    *
*-------*----------*-----------*
* 1    * 20       * 7          *
* 3    * 16.50    * 4          *
*------------------------------*

mud examples/data.csv filter id lte 150 -c 10 -r ;

========*========*========*========*=========
     id * value1 * value2 * value3 *  value4
========*========*========*========*=========
 150.00 *  25.60 *  71.00 *  69.04 * 4321.00
 149.00 *  92.37 * 748.00 *  36.71 * 1098.00
 148.00 *  69.04 * 415.00 *   3.48 * 8765.00
 147.00 *  36.71 *  82.00 *  70.15 * 5432.00
 146.00 *   3.48 * 759.00 *  47.82 * 2109.00
 145.00 *  70.15 * 426.00 *  14.59 * 9876.00
 144.00 *  47.82 * 193.00 *  81.26 * 6543.00
 143.00 *  14.59 * 760.00 *  58.93 * 3210.00
 142.00 *  81.26 * 437.00 *  25.60 * 1987.00
 141.00 *  58.93 * 104.00 *  92.37 * 7654.00
========*========*========*========*=========


mud examples/data.csv mean -x id -x value2;

========*========*=========
 value1 * value3 *  value4
========*========*=========
  49.63 *  48.75 * 5115.49
========*========*=========


 mud median -x id < examples/data.csv;

========*========*========*=========
 value1 * value2 * value3 *  value4
========*========*========*=========
  42.26 * 737.50 *  80.71 * 5987.50
========*========*========*=========


mud json -o examples/a.json < examples/a.csv;

[
  {
    "offset": 10.0,
    "count": 1.0
  },
  {
    "count": 2.0,
    "offset": 9.0
  },
  {
    "offset": 8.0,
    "count": 3.0
  },
  {
    "count": 4.0,
    "offset": 7.0
  }
]
```

The project is licensed under the [MIT](LICENSE) License.
