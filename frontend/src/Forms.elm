module Forms exposing (..)

import FeatherIcons as FI
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)


inputText : String -> (String -> msg) -> String -> Html msg
inputText label toMsg value =
    div []
        [ Html.label [ for label ] [ text label ]
        , input
            [ type_ "text"
            , placeholder label
            , Html.Attributes.value value
            , Html.Events.onInput toMsg
            , id label
            ]
            []
        ]


button : String -> msg -> Html msg
button label toMsg =
    a [ href "#", Html.Events.onClick toMsg ] [ text label ]


iconButton : FI.Icon -> msg -> Html msg
iconButton icon toMsg =
    a [ href "#", Html.Events.onClick toMsg ] [ FI.toHtml [] icon ]


row : List (Html msg) -> Html msg
row =
    tr []
        << List.map (\cell -> td [] [ cell ])


displayRowAdd : msg -> List String -> Html msg
displayRowAdd toMsg list =
    row <| rowElements list ++ [ iconButton FI.plusSquare toMsg ]


rowElements : List String -> List (Html msg)
rowElements list =
    List.map (\text -> span [] [ Html.text text ]) list


displayRow : msg -> msg -> List String -> Html msg
displayRow editMsg deleteMsg list =
    row <| rowElements list ++ [ iconButton FI.edit editMsg, iconButton FI.trash2 deleteMsg ]
