module RecipesList exposing (..)

import Element exposing (..)
import Element.Background
import Element.Border
import Element.Input
import Html exposing (a)
import Http
import Json.Decode as Decode
import Platform.Cmd as Cmd
import Recipes.Model exposing (RecipeMsg)
import Test.ExpandableList as ExpandableList exposing (ExpandableList, ExpandableListMsg, mapElementMsg)
import Test.SearchDropdown exposing (searchDropdown)
import Test.Styles exposing (white)
import WebData exposing (RemoteData(..), WebData)
import Test.StringUtils as StringUtils


type alias RecipesList =
    { recipes : WebData (ExpandableList Recipe RecipeListMsg RecipeMsg)
    , ingredients : WebData (List MetaIngredient)
    , units : WebData (List Unit)
    }


type Recipes
    = Recipes RecipesList


type alias RecipeData =
    { name : String
    , id : Maybe Int
    , comment : Maybe String
    , ingredients : WebData (List RecipeIngredient)
    }


type alias RecipeIngredient =
    { ingredient : MetaIngredient
    , unit : Unit
    , amount : String
    , allIngredients : { list : WebData (List MetaIngredient), search : String, hidden : Bool }
    , allUnits : { list : WebData (List Unit), search : String, hidden : Bool }
    }


type MetaIngredient
    = Ingredient
        { id : Int
        , name : String
        }
    | Subrecipe { id : Int, name : String }


type alias Unit =
    { id : Int
    , name : String
    }


type Recipe
    = Recipe { data : RecipeData, edit : RecipeData }


type RecipeListMsg
    = ListMsg (ExpandableListMsg Recipe RecipeMsg)
    | GotWebData WebDataMsg


type WebDataMsg
    = GotRecipes (Result Http.Error (List Recipe))
    | GotRecipeIngredients Int (Result Http.Error (List RecipeIngredient))
    | GotMetaingredients (Result Http.Error (List MetaIngredient))
    | GotUnits (Result Http.Error (List Unit))


type RecipeMsg
    = NameChange String
    | CommentChange String
    | RecipeIngredientChange RecipeIngredient RecipeIngredientMsg
    | Save
    | Cancel
    | FetchIngredients


type RecipeIngredientMsg
    = AmountChanged String
    | IngredientChanged MetaIngredient
    | IngredientFilterChange String
    | IngredientFocus
    | UnitChanged Unit
    | UnitFilterChange String
    | UnitFocus


stateOf : String -> List ( Bool, Recipe ) -> WebData (ExpandableList Recipe RecipeListMsg RecipeMsg)
stateOf search items =
    let
        filter : String -> Recipe -> Bool
        filter string ingredient =
            case ingredient of
                Recipe { data } ->
                    String.contains (String.toLower string) (String.toLower data.name)
    in
    Success
        { search = search
        , filter = filter
        , items = items
        , viewElement = viewRecipe
        , mapMsg = ListMsg
        , update = updateRecipe
        , add =
            Just
                (\() ->
                    newRecipe Nothing "" (Just "")
                )
        , expandItem =
            Just <|
                \recipe ->
                    case recipe of
                        Recipe { data } ->
                            Maybe.withDefault Cmd.none <| Maybe.map fetchRecipeIngredients data.id
        }


emptyRecipesData : RecipesList
emptyRecipesData =
    { recipes = NotAsked
    , ingredients = NotAsked
    , units = NotAsked
    }


newRecipe : Maybe Int -> String -> Maybe String -> Recipe
newRecipe id name comment =
    let
        data =
            { id = id, name = name, comment = comment, ingredients = NotAsked }
    in
    Recipe { data = data, edit = data }



-- Views


view : RecipesList -> Element RecipeListMsg
view model =
    case model.recipes of
        Success data ->
            ExpandableList.view data

        Failure _ ->
            el [] (text "Failed to load ingredients")

        _ ->
            el [] (text "Loading")


viewRecipe : Attribute RecipeListMsg -> Bool -> Recipe -> Element RecipeListMsg
viewRecipe expand expanded recipe =
    case recipe of
        Recipe { data } ->
            column [ width fill ]
                [ el [ expand, width fill ] (viewRow data)
                , if expanded then
                    viewExpanded recipe

                  else
                    none
                ]


viewRow : RecipeData -> Element msg
viewRow data =
    row [ spaceEvenly, width fill, paddingXY 50 20 ]
        [ el [ width (fillPortion 1) ] (text (Maybe.withDefault "" (Maybe.map String.fromInt data.id)))
        , el [ width (fillPortion 4) ] (text data.name)
        , el [ width (fillPortion 5) ] (text (Maybe.withDefault "" data.comment))
        ]


viewExpanded : Recipe -> Element RecipeListMsg
viewExpanded recipe =
    case recipe of
        Recipe { edit } ->
            column [ Element.Background.color white, width fill, padding 10, spacing 10, Element.Border.rounded 5 ]
                [ Element.map (ListMsg << ExpandableList.mapElementMsg recipe)
                    (Element.Input.text []
                        { onChange = NameChange
                        , label = Element.Input.labelLeft [] (text "Name:")
                        , placeholder = Just (Element.Input.placeholder [] (text "Name"))
                        , text = edit.name
                        }
                    )
                , Element.map
                    (ListMsg << ExpandableList.mapElementMsg recipe)
                    (Element.Input.text []
                        { onChange = CommentChange
                        , label = Element.Input.labelLeft [] (text "Comment:")
                        , placeholder = Just (Element.Input.placeholder [] (text "Comment"))
                        , text = Maybe.withDefault "" edit.comment
                        }
                    )
                , Element.map (ListMsg << mapElementMsg recipe) <| viewIngredients edit.ingredients
                , Element.map
                    (ListMsg << ExpandableList.mapElementMsg recipe)
                    (row [ width fill, spacing 25 ]
                        [ Element.Input.button [ alignRight ] { onPress = Just Save, label = el [ padding 10 ] <| text "Save" }
                        , Element.Input.button [ alignRight ] { onPress = Just Cancel, label = el [ padding 10 ] <| text "Cancel" }
                        ]
                    )
                ]


viewRecipeIngredient : RecipeIngredient -> Element RecipeIngredientMsg
viewRecipeIngredient recipeIngredient =
    let
        nameOf a =
            case a of
                Ingredient { name } ->
                    name

                Subrecipe { name } ->
                    name
    in
    case ( recipeIngredient.allIngredients.list, recipeIngredient.allUnits.list ) of
        ( Success i, Success u ) ->
            row [ width fill, padding 20 ]
                [ el [ width (fillPortion 3) ]
                    (searchDropdown
                        { search = recipeIngredient.allIngredients.search
                        , filter = \s a -> StringUtils.fuzzyContains (nameOf a) s
                        , items = i
                        , select = IngredientChanged
                        , selection = List.head (List.filter (\e -> e == recipeIngredient.ingredient) i)
                        , viewItem = text << nameOf
                        , hidden = recipeIngredient.allIngredients.hidden
                        , filterChange = IngredientFilterChange
                        , onFocus = IngredientFocus
                        }
                    )
                , el [ width (fillPortion 3) ] (text recipeIngredient.amount)
                , el [ width (fillPortion 1) ] (text recipeIngredient.unit.name)
                ]

        _ ->
            row [ width fill, padding 20 ]
                [ el [ width (fillPortion 3) ] (text <| nameOf recipeIngredient.ingredient)
                , el [ width (fillPortion 3) ] (text recipeIngredient.amount)
                , el [ width (fillPortion 1) ] (text recipeIngredient.unit.name)
                ]


viewIngredients : WebData (List RecipeIngredient) -> Element RecipeMsg
viewIngredients wd =
    case wd of
        Failure _ ->
            text "Failure loading ingredients"

        Success list ->
            column [ width fill ]
                (List.map (\e -> Element.map (RecipeIngredientChange e) (viewRecipeIngredient e)) list)

        _ ->
            text "Loading ingredients"



-- Updates


replaceId : (( Bool, Recipe ) -> ( Bool, Recipe )) -> Int -> List ( Bool, Recipe ) -> List ( Bool, Recipe )
replaceId f id list =
    List.map
        (\( expanded, r ) ->
            case r of
                Recipe { data } ->
                    if data.id == Just id then
                        f ( expanded, r )

                    else
                        ( expanded, r )
        )
        list


updateRecipeIngredient : RecipeIngredientMsg -> RecipeIngredient -> RecipeIngredient
updateRecipeIngredient msg ri =
    case msg of
        AmountChanged s ->
            Debug.todo "Parse number"

        IngredientChanged ingredient ->
            { ri | ingredient = ingredient }

        IngredientFocus ->
            let
                all =
                    ri.allIngredients
            in
            { ri | allIngredients = { all | hidden = not all.hidden } }

        IngredientFilterChange search ->
            let
                all =
                    ri.allIngredients
            in
            { ri | allIngredients = { all | search = search } }

        UnitFocus ->
            let
                all =
                    ri.allUnits
            in
            { ri | allUnits = { all | hidden = not all.hidden } }

        UnitFilterChange search ->
            let
                all =
                    ri.allUnits
            in
            { ri | allUnits = { all | search = search } }

        UnitChanged unit ->
            { ri | unit = unit }


updateRecipe : RecipeMsg -> Recipe -> ( Recipe, Cmd RecipeListMsg )
updateRecipe msg rc =
    let
        updateEdit f =
            case rc of
                Recipe i ->
                    let
                        edit =
                            i.edit
                    in
                    Recipe { i | edit = f edit }
    in
    case msg of
        NameChange name ->
            ( updateEdit (\e -> { e | name = name }), Cmd.none )

        CommentChange comment ->
            ( updateEdit (\e -> { e | comment = Just comment }), Cmd.none )

        Save ->
            ( rc, Cmd.none )

        Cancel ->
            case rc of
                Recipe { data } ->
                    ( Recipe { data = data, edit = data }, Cmd.none )

        RecipeIngredientChange ri riMsg ->
            let
                updateIfSame =
                    List.map
                        (\r ->
                            if r == ri then
                                updateRecipeIngredient riMsg ri

                            else
                                r
                        )
            in
            case rc of
                Recipe { data, edit } ->
                    ( Recipe
                        { data = data
                        , edit = { edit | ingredients = WebData.map updateIfSame edit.ingredients }
                        }
                    , Cmd.none
                    )

        FetchIngredients ->
            case rc of
                Recipe { data } ->
                    ( rc, Maybe.withDefault Cmd.none <| Maybe.map fetchRecipeIngredients data.id )


handleWebData : WebDataMsg -> RecipesList -> ( RecipesList, Cmd RecipeListMsg )
handleWebData msg model =
    let
        saveUnits : WebData (List Unit) -> RecipeIngredient -> RecipeIngredient
        saveUnits wd ri =
            { ri | allUnits = { list = wd, search = ri.allUnits.search, hidden = True } }

        saveMetaIngredients : WebData (List MetaIngredient) -> RecipeIngredient -> RecipeIngredient
        saveMetaIngredients wd ri =
            { ri | allIngredients = { list = wd, search = ri.allUnits.search, hidden = True } }

        saveInRecipe : (RecipeIngredient -> RecipeIngredient) -> Recipe -> Recipe
        saveInRecipe f r =
            case r of
                Recipe { edit, data } ->
                    case edit.ingredients of
                        Success ig ->
                            Recipe { data = data, edit = { edit | ingredients = Success <| List.map f ig } }

                        _ ->
                            Recipe { data = data, edit = edit }

        noop =
            ( model, Cmd.none )
    in
    case ( msg, model.recipes ) of
        ( GotRecipes result, Success list ) ->
            case result of
                Ok new ->
                    ( { model
                        | recipes = Success { list | items = List.map (Tuple.pair False) new }
                      }
                    , Cmd.none
                    )

                Err e ->
                    ( { model | recipes = Failure e }, Cmd.none )

        ( GotRecipes result, _ ) ->
            case result of
                Ok new ->
                    ( { model | recipes = stateOf "" <| List.map (Tuple.pair False) new }, Cmd.none )

                Err e ->
                    ( { model | recipes = Failure e }, Cmd.none )

        ( GotRecipeIngredients id result, Success list ) ->
            let
                ingredients =
                    case result of
                        Ok i ->
                            Success i

                        Err e ->
                            Failure e

                saveIngredients =
                    \( e, r ) ->
                        case r of
                            Recipe { data, edit } ->
                                ( e, Recipe { data = { data | ingredients = ingredients }, edit = { edit | ingredients = ingredients } } )
            in
            ( { model | recipes = Success { list | items = replaceId saveIngredients id list.items } }
            , Cmd.batch
                [ fetchAllMetaIngredients
                , fetchUnits
                ]
            )

        ( GotMetaingredients result, Success list ) ->
            case result of
                Ok meta ->
                    ( { model
                        | recipes = Success { list | items = List.map (Tuple.mapSecond <| saveInRecipe <| saveMetaIngredients <| Success meta) list.items }
                        , ingredients = Success meta
                      }
                    , Cmd.none
                    )

                Err _ ->
                    noop

        ( GotUnits result, Success list ) ->
            case result of
                Ok units ->
                    ( { model
                        | recipes = Success { list | items = List.map (Tuple.mapSecond <| saveInRecipe <| saveUnits <| Success units) list.items }
                        , units = Success units
                      }
                    , Cmd.none
                    )

                Err _ ->
                    noop

        ( GotRecipeIngredients _ _, _ ) ->
            noop

        ( GotMetaingredients _, _ ) ->
            noop

        ( GotUnits _, _ ) ->
            noop


update : RecipeListMsg -> RecipesList -> ( RecipesList, Cmd RecipeListMsg )
update msg model =
    case ( msg, model.recipes ) of
        ( ListMsg m, Success data ) ->
            Tuple.mapFirst
                (\result -> { model | recipes = Success result })
                (ExpandableList.update m data)

        ( GotWebData wd, _ ) ->
            handleWebData wd model

        _ ->
            ( model, Cmd.none )



-- Decoding


decodeUnit : Decode.Decoder Unit
decodeUnit =
    Decode.map2 Unit
        (Decode.field "unit_id" Decode.int)
        (Decode.field "name" Decode.string)


decodeRecipe : Decode.Decoder Recipe
decodeRecipe =
    Decode.map3 newRecipe
        (Decode.field "recipe_id" (Decode.nullable Decode.int))
        (Decode.field "name" Decode.string)
        (Decode.field "comment" (Decode.nullable Decode.string))


decodeNestedWeightedMetaIngredients : Decode.Decoder (List RecipeIngredient)
decodeNestedWeightedMetaIngredients =
    Decode.list decodeNestedWeightedMetaIngredient


decodeNestedWeightedMetaIngredient : Decode.Decoder RecipeIngredient
decodeNestedWeightedMetaIngredient =
    Decode.map3
        (\i a u ->
            { ingredient = i
            , amount = a
            , unit = u
            , allIngredients = { list = NotAsked, search = "", hidden = True }
            , allUnits = { list = NotAsked, search = "", hidden = True }
            }
        )
        (Decode.field "ingredient" decodeMetaIngredient)
        (Decode.field "amount" Decode.string)
        (Decode.field "unit" decodeUnit)


decodeMetaIngredient : Decode.Decoder MetaIngredient
decodeMetaIngredient =
    Decode.oneOf
        [ Decode.field "Ingredient" <| Decode.map2 (\id name -> Ingredient { id = id, name = name }) (Decode.field "ingredient_id" Decode.int) (Decode.field "name" Decode.string)
        , Decode.field "MetaRecipe" <| Decode.map2 (\id name -> Subrecipe { id = id, name = name }) (Decode.field "recipe_id" Decode.int) (Decode.field "name" Decode.string)
        ]


decodeMetaIngredients : Decode.Decoder (List MetaIngredient)
decodeMetaIngredients =
    Decode.list decodeMetaIngredient


decodeRecipes : Decode.Decoder (List Recipe)
decodeRecipes =
    Decode.list decodeRecipe


fetchRecipes : Cmd RecipeListMsg
fetchRecipes =
    Http.get
        { url = "http://localhost:3000/recipes/list"
        , expect = Http.expectJson (GotWebData << GotRecipes) decodeRecipes
        }


fetchRecipeIngredients : Int -> Cmd RecipeListMsg
fetchRecipeIngredients recipeId =
    Http.get
        { url = "http://localhost:3000/recipes/" ++ String.fromInt recipeId ++ "/meta_ingredients/list"
        , expect = Http.expectJson (GotWebData << GotRecipeIngredients recipeId) decodeNestedWeightedMetaIngredients
        }


fetchAllMetaIngredients : Cmd RecipeListMsg
fetchAllMetaIngredients =
    Http.get
        { url = "http://localhost:3000/recipes/meta_ingredients/list"
        , expect = Http.expectJson (GotWebData << GotMetaingredients) decodeMetaIngredients
        }


fetchUnits : Cmd RecipeListMsg
fetchUnits =
    Http.get
        { url = "http://localhost:3000/utils/units"
        , expect = Http.expectJson (GotWebData << GotUnits) (Decode.list decodeUnit)
        }
