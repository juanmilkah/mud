# Mud

Visualize data from a file with comma separated values.

```bash
id,price,quantity
1,20,7
2,15,3
3,16.50,4
```

```bash
mud --sort "price";

*------------------------------*
* id    * price    * amount    *
*-------*----------*-----------*
* 1     * 15.00    * 3         *
* 2     * 16.50    * 4         *
* 3     * 20       * 7         *
*------------------------------*

```

The project is licensed under the [MIT](LICENSE) License.
