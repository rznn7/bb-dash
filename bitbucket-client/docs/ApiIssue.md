# ApiIssue

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**r#type** | **String** |  | 
**links** | Option<**serde_json::Value**> |  | [optional]
**id** | Option<**i32**> |  | [optional]
**repository** | Option<[**models::ApiRepository**](Repository.md)> |  | [optional]
**title** | Option<**String**> |  | [optional]
**reporter** | Option<[**models::ApiAccount**](Account.md)> |  | [optional]
**assignee** | Option<[**models::ApiAccount**](Account.md)> |  | [optional]
**created_on** | Option<**String**> |  | [optional]
**updated_on** | Option<**String**> |  | [optional]
**edited_on** | Option<**String**> |  | [optional]
**state** | Option<**State**> |  (enum: submitted, new, open, resolved, on hold, invalid, duplicate, wontfix, closed) | [optional]
**kind** | Option<**Kind**> |  (enum: bug, enhancement, proposal, task) | [optional]
**priority** | Option<**Priority**> |  (enum: trivial, minor, major, critical, blocker) | [optional]
**milestone** | Option<[**models::ApiMilestone**](Milestone.md)> |  | [optional]
**version** | Option<[**models::ApiVersion**](Version.md)> |  | [optional]
**component** | Option<[**models::ApiComponent**](Component.md)> |  | [optional]
**votes** | Option<**i32**> |  | [optional]
**content** | Option<**serde_json::Value**> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


