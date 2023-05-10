module Cursor exposing (..)

{-| A cursor is a pointer to an element in a list. It is used to keep track of
which element is currently active in a list.
-}

import Html exposing (a)


type alias Cursor a =
    { left : List a
    , active : a
    , right : List a
    }


{-| Create a cursor from a list. The active element is set to the first element in the list.
-}
create : a -> List a -> Cursor a
create a r =
    { left = []
    , active = a
    , right = r
    }


{-| Get the active element in the list. If the list is empty, Nothing is returned.
-}
active : Cursor a -> a
active cursor =
    cursor.active


{-| Move the active element to the indexed element in the list. If the index is out of bounds, the cursor is not moved.
-}
setActive : Int -> Cursor a -> Cursor a
setActive index cursor =
    let

        l =
            List.take index (list cursor)

        r =
            List.drop (index + 1) (list cursor)

        newActive =
            List.drop index (list cursor) |> List.head
    in
    case newActive of
        Just a ->
            { cursor | left = l, active = a, right = r }

        Nothing ->
            cursor


{-| Set the active element to the first matching element in the list. If none matches, the same sursor is returned
-}
setActiveBy : (a -> Bool) -> Cursor a -> Cursor a
setActiveBy f cursor =
    let
        indexed =
            List.indexedMap (\i a -> ( i, a )) (list cursor)
    in
    case List.filter (f << Tuple.second) indexed of
        [] ->
            cursor

        ( i, _ ) :: _ ->
            setActive i cursor


{-| Get the list of elements to the left of the active element.
-}
left : Cursor a -> List a
left cursor =
    cursor.left


{-| Get the list of elements to the right of the active element.
-}
right : Cursor a -> List a
right cursor =
    cursor.right

list : Cursor a -> List a
list cursor =
    cursor.left ++ cursor.active :: cursor.right


modifyAt : Int -> (a -> a) -> Cursor a -> Cursor a
modifyAt index f cursor =
    let
        lenLeft =
            List.length cursor.left

        lenRight =
            List.length cursor.right

        len =
            lenLeft + lenRight + 1

        mapper =
            \m i a ->
                if i == m then
                    f a

                else
                    a
    in
    if index < 0 || index > len then
        cursor

    else if index < lenLeft then
        { cursor | left = List.indexedMap (mapper index) cursor.left }

    else if index == lenLeft then
        { cursor | active = f cursor.active }

    else
        { cursor | right = List.indexedMap (mapper (index - lenLeft - 1)) cursor.right }
