from setuptools import setup, find_packages

setup(name='ppsdf',
    version='1.0',
    description='Parser for CS:GO Demo files',
    author='LaihoE',
    packages=find_packages(),

    install_requires=[
    "numpy",
    "polars",
    "pandas",
    "pyarrow"
    ],
    )