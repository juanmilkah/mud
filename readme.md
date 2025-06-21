# Mud

Visualize data from a file with comma separated values.

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
```

The project is licensed under the [MIT](LICENSE) License.
