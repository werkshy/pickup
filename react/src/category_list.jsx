import React, { useEffect, useState } from "react";
import { Link } from "react-router-dom";

function Category(props) {
  let target = `/category/${props.category}`;
  return (
    <li>
      <span className="title" data-name="{props.category}">
        <Link to={target}>{props.category}</Link>
      </span>
    </li>
  );
}

export function CategoryList() {
  const [categories, setCategories] = useState([]);

  const fetchData = async () => {
    const response = await fetch("/api/categories/");
    const categories = await response.json();
    console.log("Categories: ", categories);
    setCategories(categories);
  };

  // FIXME we should use a shared global state for categories
  useEffect(() => {
    fetchData();
  }, []);

  return (
    <ul className="categories">
      <li className="header">Categories:</li>
      {categories.map((category) => (
        <Category category={category.Name} key={category.Name} />
      ))}
    </ul>
  );
}
